#!perl
use strict;
use warnings;

require Data::Dump;
use lib "/home/kent/rust/vcheck/lib/";
use lib "/home/kent/perl/kentnl/JSON-PrettyCompact/lib";
use resultdb;

my $rdb = resultdb->new();

our $VERSION_BASE = $rdb->root();
our $TEMPLATE     = "${VERSION_BASE}/index.html.tpl";
our $TARGET       = "${VERSION_BASE}/index.html";

sub find_crates {
    grep {
              -r $rdb->crate_vjson_path($_)
          and -r $rdb->crate_dir($_)
          . '/index.html'
    } $rdb->crate_names;
}

sub parse_version_info {
    my ($crate) = @_;
    my $info_hash = {
        num_fail      => 0,
        num_pass      => 0,
        num_results   => 0,
        num_versions  => 0,
        rustc_index   => {},
        version_info  => {},
        version_order => [],
    };
    my $crateinfo = $rdb->crate_info($crate);
    my $crate_dir = "${VERSION_BASE}/${crate}";

    # Get baseline data
    for my $version ( @{ $crateinfo->versions } ) {
        next if $crateinfo->is_yanked($version);
        push @{ $info_hash->{version_order} }, $version;
        if ( not exists $info_hash->{version_info}->{$version} ) {
            $info_hash->{version_info}->{$version} = {
                num_fail    => 0,
                num_pass    => 0,
                num_results => 0,
                rustc       => {},
            };
            $info_hash->{num_versions}++;
        }
        if ( $crateinfo->is_yanked($version) ) {
            $info_hash->{version_info}->{$version}->{message} = 'YANKED';
        }
    }

    for my $rustc ( @{ $crateinfo->rustcs } ) {
        $info_hash->{rustc_index}->{$rustc} = 1;
        for my $result ( @{ $crateinfo->rustc_results($rustc) } ) {
            my $version = $result->[0];
            defined $version and length $version
              or die "Bad entry " . Data::Dump::pp($result);
            my $message = $result->[1];

            $info_hash->{num_results}++;
            $info_hash->{version_info}->{$version}->{num_results}++;
            if ( defined $message ) {
                if ( $message eq "pass" ) {
                    $info_hash->{num_pass}++;
                    $info_hash->{version_info}->{$version}->{num_pass}++;
                    $info_hash->{version_info}->{$version}->{rustc}->{$rustc} =
                      "pass";
                }
                elsif ( $message eq "fail" ) {
                    $info_hash->{num_fail}++;
                    $info_hash->{version_info}->{$version}->{num_fail}++;
                    $info_hash->{version_info}->{$version}->{rustc}->{$rustc} =
                      "fail";
                }
            }
        }
    }

    return $info_hash;
}

sub pick_semver_vmax {
    my (@versions) = @_;
    my %vcache;
    my (@out);
    for my $version (@versions) {
        my ($vparts) = [ split /[.]/, $version ];
        my $vmajor   = $vparts->[0];
        my $vminor   = $vparts->[1];
        next if $vmajor eq '0' and $vminor eq '0';
        my $cache_key = join '.', $vmajor, $vminor;
        next if exists $vcache{$cache_key};
        $vcache{$cache_key} = $version;
        push @out, $version;
    }
    return @out;
}

sub gen_crate_report {
    my ($crate) = @_;
    my $pad     = " " x 6;
    my $buffer  = "";
    open my $fh, ">", \$buffer or die "Can't open buffer for write, $!";
    $fh->printf(
        "$pad<li><span class=\"cratename\"><a href=\"./%s\">%s</a></span>",
        $crate, $crate );
    my $info      = parse_version_info($crate);
    my $crateinfo = $rdb->crate_info($crate);
    if ( $info->{num_pass} > 1 and $info->{num_fail} == 0 ) {
        $fh->print(
"<span class=\"grade goldstar\" title=\"No reported failures for any version on any rust\">&#x1F31F;</span>"
        );
    }
    else {
        my $fail_pct = sprintf "%0.1f",
          $info->{num_fail} / $info->{num_results};
        $fh->printf(
            "<span class=\"grade numeric_%d\" title=\"%d%% failures\"></span>",
            $fail_pct * 10,
            $fail_pct * 100
        );
    }
    $fh->print("<span class=\"extlinks\">");
    $fh->printf(
        "<a href=\"https://crates.io/crates/%s\">crates.io&#x1F517;</a>",
        $crate );
    $fh->printf( "<a href=\"https://lib.rs/crates/%s\">lib.rs&#x1F517;</a>",
        $crate );
    $fh->printf( "<a href=\"https://docs.rs/%s\">docs.rs&#x1F517;</a>",
        $crate );
    $fh->printf("</span>");
    if ( $info->{num_pass} > 1 ) {
        my (@vpick) = pick_semver_vmax( reverse @{ $info->{version_order} } );
        if (@vpick) {
            $fh->print("\n");
            $fh->print("$pad  <ul>\n");
            for (@vpick) {
                my $v_result = $info->{version_info}->{$_};
                $fh->printf( "$pad    <li><span class=\"version\">%s</span>",
                    $_ );
                my $min_test_rustc = [ $rdb->rustc_order ]->[0];
                my $max_test_rustc = [ $rdb->rustc_order ]->[-1];

                my $min_rustc = $crateinfo->srv_min_version($_);
                my $max_rustc = $crateinfo->srv_max_version($_);

                if ( not defined $min_rustc ) {
                    $fh->print(
"<span class=\"msrv unknown_msrv\" title=\"no known working rust version\">Unusable</span>"
                    );
                }
                elsif ($min_rustc ne $min_test_rustc
                    or $max_rustc ne $max_test_rustc )
                {
                    my $msrv = join q[.],
                      splice @{ [ split /[.]/, $min_rustc ] },
                      0, 2;
                    my $mxrv = join q[.],
                      splice @{ [ split /[.]/, $max_rustc ] },
                      0, 2;
                    if ( $min_rustc eq $min_test_rustc ) {
                        $fh->printf(
"<span class=\"msrv max_msrv\" title=\"only works on rust versions &lt;= %s\">MaxSRV %s</span>",
                            $mxrv, $mxrv
                        );

                    }
                    elsif ( $max_rustc eq $max_test_rustc ) {
                        $fh->printf(
"<span class=\"msrv min_msrv\" title=\"only works on rust versions &gt;= %s\">MSRV %s</span>",
                            $msrv, $msrv,
                        );
                    }
                    else {
                        $fh->printf(
"<span class=\"msrv between_msrv\" title=\"only works on rust versions &gt;= %s, &lt;= %s\">MSRV %s, MaxSRV %s</span>",
                            $msrv, $mxrv, $msrv, $mxrv, );

                    }
                }
                if ( $v_result->{num_pass} > 1 and $v_result->{num_fail} == 0 )
                {
                    $fh->print(
"<span class=\"grade goldstar\" title=\"No reported failures for this version on any rust\">&#x1F31F;</span>"
                    );
                }
                elsif ( $v_result->{num_results} == 0 ) {
                    warn "No results for version $_ of $crate!";
                    next;
                }
                else {
                    my $fail_pct = sprintf "%0.1f",
                      $v_result->{num_fail} / $v_result->{num_results};
                    $fh->printf(
"<span class=\"grade numeric_%d\" title=\"%d%% failures\"></span>",
                        $fail_pct * 10,
                        $fail_pct * 100
                    );
                }
                $fh->print("</li>\n");
            }
            $fh->print("$pad  </ul>\n");
            $fh->print("$pad");
        }
    }
    $fh->print("</li>\n");
    close $fh or warn "Can't close buffer, $!";
    return $buffer;
}

sub gen_section {
    my ( $label, $members ) = @_;
    my $pad    = " " x 4;
    my $buffer = "";
    open my $fh, ">", \$buffer or die "Can't open buffer for write, $!";
    $fh->printf(
        "$pad<h2 id=\"crate-%s\">%s*<a href=\"#crate-%s\">&#x2693;</a></h2>\n",
        $label, $label, $label );
    $fh->printf("$pad<ul>\n");
    for my $crate ( sort @{$members} ) {
        $fh->print( gen_crate_report($crate) );
    }
    $fh->printf("$pad</ul>\n");
    close $fh or warn "Error closing buffer, $!";
    return $buffer;
}

sub vsort {
    my ( $lhs, $rhs ) = @_;
    my (@lhs_parts) = split /[.]/, $lhs;
    my (@rhs_parts) = split /[.]/, $rhs;

    $lhs_parts[0] <=> $rhs_parts[0]
      or $lhs_parts[1] <=> $rhs_parts[1]
      or $lhs_parts[2] <=> $rhs_parts[2];
}

sub gen_toc {
    my %crate_buckets;
    for my $crate (find_crates) {
        my $bucket = substr $crate, 0, 1;
        push @{ $crate_buckets{$bucket} }, $crate;
    }

    my $inject = "";

    open my $fh, ">", \$inject or die "Can't open buffer for write";

    my $pad = " " x 4;
    for my $bucket ( sort keys %crate_buckets ) {
        $fh->print( gen_section( $bucket, $crate_buckets{$bucket} ) );
    }
    close $fh or warn "Error closing buffer";
    return $inject;
}

my $code = do {
    open my $fh, "<", $TEMPLATE or die "Can't read $TEMPLATE, $!";
    local $/ = undef;
    scalar <$fh>;
};
use JSON::MaybeXS;
my $jxs         = JSON::MaybeXS->new();
my $projections = $jxs->decode(
    do {
        my $rfile = $rdb->root() . '/projections.json';
        open my $fh, '<:utf8', $rfile or die "Can't read $rfile, $!";
        local $/;
        scalar <$fh>;
    }
);
$code =~ s{^\s*[<]!--\s*build\s+reports\s*--[>]\s*\n}{gen_toc}gmsex;
$code =~ s{\#\[projection_([^\]]+)\]}{
  $projections->{$1}
}gmsex;
open my $fh, ">", $TARGET or die "Can't write $TEMPLATE, $!";
$fh->print($code);
close $fh or warn "error closing $TEMPLATE, $!";
