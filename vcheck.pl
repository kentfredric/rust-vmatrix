#!perl
use strict;
use warnings;

use Test::TempDir::Tiny qw( in_tempdir );
use Capture::Tiny qw( capture );

use lib "/home/kent/rust/vcheck/lib/";
use lib "/home/kent/perl/kentnl/JSON-PrettyCompact/lib";
use resultdb;

our $CRATE = "libc";
$CRATE = $ENV{CRATE} if exists $ENV{CRATE} and length $ENV{CRATE};

my $rdb = resultdb->new();

my $batch_start = time;
my $test_count  = 0;

for my $rustc ( reverse $rdb->rustc_order ) {
    in_tempdir "$CRATE-rustc$rustc" => sub {
        my $cwd       = shift;
        my $crateinfo = $rdb->crate_info($CRATE);
        my (@versions) =
          grep { not $crateinfo->is_yanked($_) } @{ $crateinfo->versions };
        $test_count += scalar @versions;
        printf "entering %s\n", $cwd;

        do_testset(
            crate           => $CRATE,
            versions        => \@versions,
            rustc_version   => $rustc,
            rustc_toolchain => $rustc,
            work_dir        => $cwd,
        );
    };
}
my $batch_stop = time;
my $avg        = ( $batch_stop - $batch_start ) / $test_count;
printf
"\e[34;1m* Suite run: %d seconds for %d version targets (%4.3f seconds per target)\e[0m\n",
  ( $batch_stop - $batch_start ), $test_count, $avg;

sub do_test {
    my (%params) = @_;

    my $rustc_version   = $params{rustc_version};
    my $rustc_toolchain = $params{rustc_toolchain};

    my $crate   = $params{crate};
    my $version = $params{version};

    my $work_dir = $params{work_dir};

    my $cargo_toml = "${work_dir}/Cargo.toml";
    my $src_dir    = "${work_dir}/src";
    my $lib_path   = "${src_dir}/lib.rs";
    {
        open my $fh, ">", $cargo_toml or die "Can't write $cargo_toml";
        $fh->print(<<"EOF");
[package]
name = "test"
version = "0.1.0"
authors = [ "Kent Fredric <kentnl\@gentoo.org>" ]

[dependencies."${crate}"]
version = "=${version}"
EOF
        close $fh or warn "Error closing $cargo_toml, $!";
    }
    if ( !-d "${src_dir}" ) {
        system( "mkdir", "-p", "${src_dir}" );
    }
    if ( !-e "${lib_path}" ) {
        open my $fh, ">", $lib_path or die "Can't write $lib_path";
        $fh->print("");
        close $fh or warn "Error closing $lib_path, $!";
    }
    my $exit = system(
        "/home/kent/bin/cargo.sh", "+${rustc_toolchain}",
        "build",                   "--verbose"
    );
    if ( $exit != 0 ) {
        my $low  = $exit & 0b11111111;
        my $high = $exit >> 8;
        if ( $low == 2 ) {
            die "\e[31;1mSIGINT detected\e[0m, quitting";
        }
        printf
          "\e[31;1m>>>>\e[0m rustc %s w/ %s version %s \e[31;1mfail\e[0m\n",
          $rustc_version,
          $crate, $version;
        return undef;
    }
    printf
      "\e[32;1m>>>>\e[0m rustc %s w/ %s version %s \e[32;1mpass\e[0m\n",
      $rustc_version,
      $crate, $version;
    return 1;
}

sub do_testset {
    my (%params)        = @_;
    my $crate           = $params{crate};
    my $versions        = $params{versions};
    my $rustc_version   = $params{rustc_version};
    my $rustc_toolchain = $params{rustc_toolchain};
    my $work_dir        = $params{work_dir};

    my $start = time();

    my (@results);
    my (%prev_results);
    for
      my $prev ( @{ $rdb->crate_flat_rustc_results( $crate, $rustc_version ) } )
    {
        $prev_results{ $prev->[0] } =
          { version => $prev->[0], message => $prev->[1] };
    }
    for my $version ( reverse @$versions ) {
        if ( exists $prev_results{$version} ) {
            my $presult = $prev_results{$version};
            if ( exists $presult->{message} and length $presult->{message} ) {
                warn
"Skipping $crate v$version, previous result : $presult->{message}\n";
                push @results, $presult;
                next;
            }
        }
        my $result = do_test(
            rustc_version   => $rustc_version,
            rustc_toolchain => $rustc_toolchain,
            crate           => $crate,
            version         => $version,
            work_dir        => $work_dir,
        );
        push @results,
          { version => $version, message => ( $result ? "pass" : "fail" ) };
    }
    my $old = $rdb->crate_read_rjson($crate);
    my $jxs = $rdb->crate_read_vjson($crate);
    my $deep =
      $rdb->crate_merge_flat_rustc_results( $crate, $old, $rustc_version,
        [ map { [ $_->{version}, $_->{message} ] } reverse @results ], $jxs );
    $rdb->crate_write_rjson( $crate, $deep );
    $rdb->crate_write_flat_rustc_results( $crate, $rustc_version,
        [ reverse @results ] );
    my $stop     = time;
    my $consumed = $stop - $start;
    my $ncrates  = scalar @$versions;
    my $atime    = $consumed / $ncrates;
    printf
"\e[33;1m* processed %d versions in %d seconds (%4.2f secs per version)\e[0m\n",
      $ncrates, $consumed, $atime;

    -e "${work_dir}/target/debug"
      and system( "rm", "-r", "${work_dir}/target/debug" );
    -e "${work_dir}/target/release"
      and system( "rm", "-r", "${work_dir}/target/release" );

}

