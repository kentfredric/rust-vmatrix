#!perl
use strict;
use warnings;

require Data::Dump;

our $VERSION_BASE = "/home/kent/rust/vmatrix";
our $TEMPLATE     = "${VERSION_BASE}/index.html.tpl";
our $TARGET       = "${VERSION_BASE}/index.html";

sub find_crates {
    opendir my $dfh, $VERSION_BASE or die "Can't opendir $VERSION_BASE";
    my (@crates);
    while ( my $ent = readdir $dfh ) {
        next if $ent eq '.';
        next if $ent eq '..';
        next unless -d "${VERSION_BASE}/${ent}";
        next unless -r "${VERSION_BASE}/${ent}/versions.txt";
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
    my $crate_dir    = "${VERSION_BASE}/${crate}";
    my $version_file = "${crate_dir}/versions.txt";
    if ( !-e $version_file ) {
        die "No version file for crate $crate";
    }

    # Get baseline data
    for my $rec ( parse_vfile($version_file) ) {
        my ($version) = $rec->[0];
        defined $version and length $version
          or die "Bad entry in $version_file: " . Data::Dump::pp($rec);
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
        if ( defined $rec->[1] and length $rec->[1] ) {
            $info_hash->{version_info}->{$version}->{message} = $rec->[1];
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
    $fh->printf( "$pad<li><a href=\"./%s\">%s</a>", $crate, $crate );
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
            $fh->printf( "$pad    <li>%s", $_ );
            if ( $v_result->{num_pass} > 1 and $v_result->{num_fail} == 0 ) {
                $fh->print(
"<span class=\"grade goldstar\" title=\"No reported failures for this version on any rust\">&#x1F31F;</span>"
                );
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
