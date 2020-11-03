#!/usr/bin/env perl
use strict;
use warnings;

our $VERSION = '0.0';

use lib "/home/kent/rust/vcheck/lib/";
use lib "/home/kent/perl/kentnl/JSON-PrettyCompact/lib";
use cargo;
use resultdb;
use Time::HiRes qw(sleep);
my $rdb = resultdb->new();

my $refresh_after = 7.55 * 60 * 60;
my $poll_pause    = 0.1;
my $pause         = 1.5;
my $loop_pause    = 30;
my $min_fresh;
my $now;

if ( $ENV{WATCH} ) {
    $ENV{UPDATE} = 1;
    $ENV{QUIET}  = 1;
    *STDERR->autoflush(1);
    while (1) {
        printf "== Loop Run at %s ==\n", scalar localtime;
        $min_fresh = undef;
        $now       = time;
        do_update();
        print "\n";
        my $wait = $loop_pause;
        $wait = $min_fresh if defined $min_fresh and $min_fresh > $loop_pause;
        if ( defined $min_fresh ) {
            printf "Next update in $min_fresh seconds\n";
        }
        printf "Waiting for $wait seconds\n";
        sleep($wait);
    }
}
else {
    do_update();
}

sub do_update {
    my (@queue) =
      ( sort { ( rand() * 100 ) <=> ( rand() * 100 ) } $rdb->crate_names );
    while (@queue) {
        my $crate = shift @queue;
        next unless should_update($crate);
        my $vjs          = $rdb->crate_vjson_path($crate);
        my $old_versions = $rdb->crate_read_vjson($crate);
        my (@versions) =
          @{ cargo::update_version_info( $crate, $old_versions ) };
        my $do_write = should_write( $old_versions, \@versions );
        if ($do_write) {
            $rdb->crate_write_vjson( $crate, [ reverse @versions ] );
            warn "\e[33m*** Updated $crate ***\e[0m\n";
        }
        my (%deps) = %{ $rdb->crate_dependencies_from_json( \@versions ) };
        for my $key ( keys %deps ) {
            my $path = $rdb->root . '/' . $key;
            if ( !-d $path ) {
                warn "\e[33m New dependency: \e[32m$key\e[0m\n";
                system( 'mkdir', '-p', $path );
                unshift @queue, $key;
            }
        }

        if ( -e $vjs ) {

            # mark updated even if no updates occurred
            # because we're tracking update *attempts*
            system( 'touch', '-c', '-m', $vjs );
        }
    }
}

sub should_write {
    my ( $old, $new ) = @_;
    my (%old_vhash);
    my $yanked = sub {
        return 0 if not exists $_[0]->{yanked};
        return 0 if not $_[0]->{yanked};
        return 1;
    };
    for my $old_version ( @{$old} ) {
        $old_vhash{ $old_version->{num} } = $yanked->($old_version);
    }
    my %new_vhash;

    # detect new deps, and yank changes
    for my $new_version ( @{$new} ) {
        return 1 if not exists $old_vhash{ $new_version->{num} };
        return 1
          if $old_vhash{ $new_version->{num} } != $yanked->($new_version);
        $new_vhash{ $new_version->{num} } = 0;
    }

    # detect dep removals
    for my $old_version ( @{$old} ) {
        return 1 if not exists $new_vhash{ $old_version->{num} };
    }
    return 0;
}

sub should_update {
    my ($crate) = @_;
    my $vjs = $rdb->crate_vjson_path($crate);
    return 1 if not -e $vjs;
    return   if not $ENV{UPDATE};
    my $mtime     = [ stat $vjs ]->[9];
    my $age_secs  = $now - $mtime;
    my $freshness = sprintf "%d",
      100 - ( ( $age_secs + 0.1 ) / $refresh_after * 100 );

    my $time_till_refresh = $refresh_after - $age_secs;
    if ( $freshness > 1 and $time_till_refresh > $loop_pause ) {
        $min_fresh = $time_till_refresh if not defined $min_fresh;
        $min_fresh = $time_till_refresh if $min_fresh > $time_till_refresh;
    }
    if ( $freshness > 5 ) {
        if ( $ENV{QUIET} ) {
            *STDERR->print("\e[32m.\e[0m");
        }
        else {
            # skip entirely if its fresher than the pause limit
            warn " $crate is \e[32mfresh\e[0m ($freshness%), skipping update\n";
        }
        return;
    }
    if ( $freshness > 1 ) {
        if ( $ENV{QUIET} ) {
            *STDERR->print("\e[33m|\e[0m");
        }
        else {
            warn
              " $crate is \e[33msemifresh\e[0m($freshness%), skipping update\n";
        }
        return;
    }
    if ( $ENV{QUIET} ) {
        *STDERR->print("\e[34m^\e[0m");
    }
    else {
        warn
" $crate is \e[34mstale\e[0m($freshness%), pausing ($pause), then checking\n";
    }
    sleep($pause);
    return 1;
}
