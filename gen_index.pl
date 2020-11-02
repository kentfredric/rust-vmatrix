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
    opendir my $dfh, $VERSION_BASE or die "Can't opendir $VERSION_BASE";
    my (@crates);
    while ( my $ent = readdir $dfh ) {
        next if $ent eq '.';
        next if $ent eq '..';
        next unless -d "${VERSION_BASE}/${ent}";
        next unless -r "${VERSION_BASE}/${ent}/versions.json";
        next unless -r "${VERSION_BASE}/${ent}/index.html";
        push @crates, $ent;
    }
    return @crates;
}

sub parse_vfile {
    my ($file) = @_;
    open my $fh, "<", $file or die "Can't read $file, $!";
    map { chomp; [ split /[|]/, $_ ] } <$fh>;
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
    my $crate_dir = "${VERSION_BASE}/${crate}";

    # Get baseline data
    for my $rec ( @{ $rdb->crate_read_vjson($crate) } ) {
        my ($version) = $rec->{num};
        next if exists $rec->{yanked} and $rec->{yanked};
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
        if ( exists $rec->{yanked} and $rec->{yanked} ) {
            $info_hash->{version_info}->{$version}->{message} = 'YANKED';
        }
    }

    # Collect result data
    opendir my $dfh, $crate_dir or die "can't opendir $crate_dir";
    while ( my $ent = readdir $dfh ) {
        next if $ent eq '.';
        next if $ent eq '..';
        next unless $ent =~ /\Arustc-(\d+[.]\d+[.]\d+)\z/;
        next if -d $ent;
        my $rustc = "$1";
        my $file  = "$crate_dir/$ent";
        $info_hash->{rustc_index}->{$rustc} = 1;
        for my $result ( parse_vfile($file) ) {
            my $version = $result->[0];
            defined $version and length $version
              or die "Bad entry in $file: " . Data::Dump::pp($result);
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

    # Estimate Min/Max RustC
    my (@rustc) = sort { vsort( $a, $b ) } keys %{ $info_hash->{rustc_index} };
    for my $version ( @{ $info_hash->{version_order} } ) {
        my $vdata = $info_hash->{version_info}->{$version};
        if ( not exists $vdata->{rustc}->{ $rustc[0] }
            or $vdata->{rustc}->{ $rustc[0] } ne "pass" )
        {
            for my $rustc_ver (@rustc) {
                next unless exists $vdata->{rustc}->{$rustc_ver};
                next unless $vdata->{rustc}->{$rustc_ver} eq 'pass';
                $vdata->{min_rustc} = $rustc_ver;
                last;
            }
            if ( not exists $vdata->{min_rustc} ) {
                $vdata->{min_rustc} = 9999;
            }
        }
        if ( not exists $vdata->{rustc}->{ $rustc[-1] }
            or $vdata->{rustc}->{ $rustc[-1] } ne "pass" )
        {
            for my $rustc_ver ( reverse @rustc ) {
                next unless exists $vdata->{rustc}->{$rustc_ver};
                next unless $vdata->{rustc}->{$rustc_ver} eq 'pass';
                $vdata->{max_rustc} = $rustc_ver;
                last;
            }
            if ( not exists $vdata->{max_rustc} ) {
                $vdata->{max_rustc} = 0;
            }
        }
        if (    exists $vdata->{max_rustc}
            and $vdata->{max_rustc} eq "0"
            and exists $vdata->{min_rustc}
            and $vdata->{min_rustc} )
        {
            $vdata->{rustc_unknown} = 1;
            delete $vdata->{max_rustc};
            delete $vdata->{min_rustc};
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
    my $info = parse_version_info($crate);

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
    my (@vpick) = pick_semver_vmax( reverse @{ $info->{version_order} } );
    if (@vpick) {
        $fh->print("\n");
        $fh->print("$pad  <ul>\n");
        for (@vpick) {
            my $v_result = $info->{version_info}->{$_};
            $fh->printf( "$pad    <li><span class=\"version\">%s</span>", $_ );
            if (   exists $v_result->{min_rustc}
                or exists $v_result->{max_rustc}
                or $v_result->{rustc_unknown} )
            {
                if ( exists $v_result->{min_rustc}
                    and not exists $v_result->{max_rustc} )
                {
                    $v_result->{min_rustc} =~ /\A(\d+[.]\d+)[.]/;
                    $fh->printf(
"<span class=\"msrv min_msrv\" title=\"only works on rust versions &gt;= %s\">MSRV %s</span>",
                        $1, $1
                    );
                }
                elsif ( not exists $v_result->{min_rustc}
                    and exists $v_result->{max_rustc} )
                {
                    $v_result->{max_rustc} =~ /\A(\d+[.]\d+)[.]/;
                    $fh->printf(
"<span class=\"msrv max_msrv\" title=\"only works on rust versions &lt;= %s\">MaxSRV %s</span>",
                        $1, $1
                    );
                }
                elsif ( exists $v_result->{min_rustc}
                    and exists $v_result->{max_rustc} )
                {
                    $v_result->{min_rustc} =~ /\A(\d+[.]\d+)[.]/;
                    my $min = $1;
                    $v_result->{max_rustc} =~ /\A(\d+[.]\d+)[.]/;
                    $fh->printf(
"<span class=\"msrv between_msrv\" title=\"only works on rust versions &gt;= %s, &lt;= %s\">MSRV %s, MaxSRV %s</span>",
                        $min, $1, $min, $1, );
                }
                elsif ( $v_result->{rustc_unknown} ) {
                    $fh->print(
"<span class=\"msrv unknown_msrv\" title=\"no known working rust version\">Unusable</span>"
                    );
                }
            }
            if ( $v_result->{num_pass} > 1 and $v_result->{num_fail} == 0 ) {
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
        "$pad<h2 id=\"crate-%s\">%s*<a href=\"#crate-%s\">#</a></h2>\n",
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
$code =~ s{^\s*[<]!--\s*build\s+reports\s*--[>]\s*\n}{gen_toc}gmsex;
open my $fh, ">", $TARGET or die "Can't write $TEMPLATE, $!";
$fh->print($code);
close $fh or warn "error closing $TEMPLATE, $!";
