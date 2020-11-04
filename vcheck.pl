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

our (@RUSTC) = (
    qw(
      1.0.0  1.1.0  1.2.0  1.3.0  1.4.0  1.5.0  1.6.0  1.7.0  1.8.0  1.9.0
      1.10.0 1.11.0 1.12.1 1.13.0 1.14.0 1.15.1 1.16.0 1.17.0 1.18.0 1.19.0
      1.20.0 1.21.0 1.22.1 1.23.0 1.24.1 1.25.0 1.26.2 1.27.2 1.28.0 1.29.2
      1.30.1 1.31.1 1.32.0 1.33.0 1.34.2 1.35.0 1.36.0 1.37.0 1.38.0 1.39.0
      1.40.0 1.41.1 1.42.0 1.43.1 1.44.1 1.45.2 1.46.0 1.47.0
      )
);
my $batch_start = time;
my $test_count  = 0;

for my $rustc ( reverse @RUSTC ) {
    in_tempdir "$CRATE-rustc$rustc" => sub {
        my $cwd = shift;
        my (@versions) =
          map  { { version => $_->{num} } }
          grep { not exists $_->{yanked} or not $_->{yanked} }
          @{ $rdb->crate_read_vjson($CRATE) };
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
        if ( not defined $version->{version} ) {
            require Data::Dump;
            die "Bad line in version list: " . Data::Dump::pp($version);
        }
        if ( exists $version->{message} ) {
            warn
              "Skipping: $crate v$version->{version} : $version->{message}\n";
            push @results, $version;
            next;
        }
        if ( exists $prev_results{ $version->{version} } ) {
            my $presult = $prev_results{ $version->{version} };
            if ( exists $presult->{message} and length $presult->{message} ) {
                warn
"Skipping $crate v$version->{version}, previous result : $presult->{message}\n";
                push @results, $presult;
                next;
            }
        }
        my $result = do_test(
            rustc_version   => $rustc_version,
            rustc_toolchain => $rustc_toolchain,
            crate           => $crate,
            version         => $version->{version},
            work_dir        => $work_dir,
        );
        $version->{message} = $result ? "pass" : "fail";
        push @results, $version;
    }
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

