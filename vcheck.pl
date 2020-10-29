#!perl
use strict;
use warnings;

use Test::TempDir::Tiny qw( in_tempdir );
use Capture::Tiny qw( capture );

our $CRATE = "libc";
$CRATE = $ENV{CRATE} if exists $ENV{CRATE} and length $ENV{CRATE};

our $VERSION_BASE        = "/home/kent/rust/vmatrix";
our $VERSION_SOURCE      = "${VERSION_BASE}/${CRATE}/versions.txt";
our $VERSION_DEST_PREFIX = "${VERSION_BASE}/${CRATE}/rustc-";

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
my %used_pool;

for my $rustc ( reverse @RUSTC ) {
    if ( not $ENV{UPDATE} and -e "${VERSION_DEST_PREFIX}${rustc}" ) {
        warn "Results found for $CRATE on rustc-$rustc, skipping";
        next;
    }
    in_tempdir "$CRATE-rustc$rustc" => sub {
        my $cwd = shift;
        my (@versions) = get_versions($VERSION_SOURCE);
        $test_count += scalar @versions;
        printf "entering %s\n", $cwd;

        do_testset(
            crate           => $CRATE,
            versions        => \@versions,
            rustc_version   => $rustc,
            rustc_toolchain => $rustc,
            work_dir        => $cwd,
            result_file     => $VERSION_DEST_PREFIX . $rustc,
        );
    };
}
my $batch_stop = time;
my $avg        = ( $batch_stop - $batch_start ) / $test_count;
printf
"\e[34;1m* Suite run: %d seconds for %d version targets (%4.3f seconds per target)\e[0m\n",
  ( $batch_stop - $batch_start ), $test_count, $avg;
for ( sort keys %used_pool ) {
    printf "\e[34;1m*\e[0m Used Crate: \e[34;1m%s\e[0m\n", $_;
}

sub update_used {
    my (%params)      = @_;
    my $work_dir      = $params{work_dir};
    my $crate         = $params{crate};
    my $rustc_version = $params{rustc_version};

    my $build_dir = "${work_dir}/target/debug/build";
    my $lock_file = "${work_dir}/Cargo.lock";

    # Collect all dep names
    if ( -d $build_dir ) {
        opendir my $dfh, $build_dir or die "can't opendir $build_dir, $!";
        while ( my $ent = readdir $dfh ) {
            next if $ent eq '.';
            next if $ent eq '..';
            if ( $ent =~ /\A(.*?)-[0-9a-f]{16}\z/ ) {
                my ($dcrate) = $1;
                next if $dcrate eq $crate;
                next if exists $used_pool{$dcrate};
                $used_pool{$dcrate} = $rustc_version;
                printf "\e[33;1m*\e[0m used crate: \e[33;1m%s\e[0m\n", $dcrate;
            }
        }
    }

    # scrape cargo.lock too
    if ( -f $lock_file ) {
        open my $fh, "<", $lock_file or die "Cant' read $lock_file, $!";
        my $seen_pkg;
        while ( my $line = <$fh> ) {
            chomp $line;
            if ( $line eq '[[package]]' ) {
                $seen_pkg = 1;
                next;
            }
          ignore: {
                if ( $seen_pkg and $line =~ /\Aname = "([^"]+)"\s*\z/ ) {
                    my $dcrate = $1;
                    last ignore if $dcrate eq $crate;
                    last ignore if exists $used_pool{$dcrate};
                    last ignore if $dcrate eq 'test';
                    $used_pool{$dcrate} = $rustc_version;
                    printf "\e[33;1m*\e[0m used crate: \e[33;1m%s\e[0m\n",
                      $dcrate;
                }
            }
            $seen_pkg = undef;
        }
    }
}

sub get_versions {
    my ($path) = @_;
    open my $fh, "<", $path or die "can't read $path";
    my @v;
    while ( my $line = <$fh> ) {
        chomp $line;
        my ( $version, $message, @rest ) = split /[|]/, $line;
        my $rec = { version => $version };
        if ( defined $message and length $message ) {
            $rec->{message} = $message;
        }
        if (@rest) {
            $rec->{extras} = \@rest;
        }
        push @v, $rec;
    }
    return @v;
}

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

[dependencies.${crate}]
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
    update_used(
        work_dir      => $work_dir,
        crate         => $crate,
        rustc_version => $rustc_version,
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
    my $result_file     = $params{result_file};

    my $start = time();

    my (@results);
    my (%prev_results);
    if ( -e $result_file ) {
        warn "Loading past results from $result_file\n";
        for my $prev ( get_versions($result_file) ) {
            if ( not defined $prev->{version} ) {
                require Data::Dump;
                die "Bad line in $result_file: " . Data::Dump::pp($prev);
            }
            $prev_results{ $prev->{version} } = $prev;
        }
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
    open my $fh, ">", $result_file or die "Can't write $result_file";
    for my $result ( reverse @results ) {
        $fh->printf( "%s|%s\n", $result->{version}, $result->{message} );
    }
    close $fh or warn "Error closing $result_file, $!\n";
    my $stop     = time;
    my $consumed = $stop - $start;
    my $ncrates  = scalar @$versions;
    my $atime    = $consumed / $ncrates;
    printf
"\e[33;1m* processed %d versions in %d seconds (%4.2f secs per version)\e[0m\n",
      $ncrates, $consumed, $atime;

    for ( sort keys %used_pool ) {
        if ( $used_pool{$_} eq $rustc_version ) {
            printf
              "\e[33;1m*\e[0m Used Crate: \e[34;1m%s\e[0m (\e[32;1mNEW\e[0m)\n",
              $_;
        }
        else {
            printf
              "\e[33;1m*\e[0m Used Crate: \e[34;1m%s\e[0m (\e[32;1mNEW\e[0m)\n",
              $_;

        }
    }

    -e "${work_dir}/target/debug"
      and system( "rm", "-r", "${work_dir}/target/debug" );
    -e "${work_dir}/target/release"
      and system( "rm", "-r", "${work_dir}/target/release" );

}

