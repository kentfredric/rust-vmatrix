#!perl
use strict;
use warnings;

use lib "/home/kent/rust/vcheck/lib/";
use lib "/home/kent/perl/kentnl/JSON-PrettyCompact/lib";
use resultdb;

my $n_crates    = 0;
my $n_targets   = 0;
my $e_realistic = 7.58;
my $e_huge      = 60;

my $rdb = resultdb->new();
for my $crate ( $rdb->crate_names ) {
    my $info       = $rdb->crate_info($crate);
    my (@versions) = grep { not $info->is_yanked($_) } ( @{ $info->versions } );
    my (@untested) = @{ $info->untested_combos };
    next unless @untested;
    $n_crates++;
    for my $untested (@untested) {
        $n_targets++;
        next if $ENV{SUMMARY};
        printf "%s v=%s rustc=%s\n", $crate, $untested->{num},
          $untested->{rustc};
    }
}
if ( $ENV{SUMMARY} ) {
    require JSON::PrettyCompact;
    my $pc     = JSON::PrettyCompact->new();
    my $report = {
        at       => scalar gmtime(),
        crates   => $n_crates,
        targets  => $n_targets,
        est_low  => num_fmt( $n_targets * $e_realistic ),
        est_high => num_fmt( $n_targets * $e_huge ),
    };
    my $ds = $pc->encode($report);
    printf "At %s GMT\n",              $report->{at};
    printf "%d crates\n",              $report->{crates};
    printf "%d targets\n",             $report->{targets};
    printf "Realistic low time: %s\n", $report->{est_low};
    printf "Bad high time: %s\n",      $report->{est_high};
    my $dest = $rdb->root() . '/projections.json';
    open my $fh, ">:utf8", $dest or die "can't write $dest, $!";
    $fh->print($ds);
    close $fh or warn "error closing $dest, $!";
}

sub num_fmt {
    my ($time) = @_;
    my (@windows);
    push @windows,
      {
        max   => 0.9 * 60,
        scale => sub { $_[0] },
        fmt   => '%d seconds',
      };
    push @windows,
      {
        max   => 0.9 * 60 * 60,
        scale => sub { $_[0] / 60 },
        fmt   => '%.2f minutes',
      };
    push @windows,
      {
        max   => 0.9 * 60 * 60 * 24,
        scale => sub { $_[0] / 60 / 60 },
        fmt   => '%.2f hours',
      };
    push @windows,
      {
        max   => 0.9 * 60 * 60 * 24 * 365,
        scale => sub { $_[0] / 60 / 60 / 24 },
        fmt   => '%.2f days',
      };
    push @windows,
      {
        max   => $time + 1,
        scale => sub { $_[0] / 60 / 60 / 24 / 365 },
        fmt   => '%.2f years',
      };
    my $prev_window = undef;

    for my $window (@windows) {
        my $this_window = sprintf $window->{fmt}, $window->{scale}->($time);
        if ( defined $prev_window ) {
            $prev_window = $this_window . " ( " . $prev_window . " )";
        }
        else {
            $prev_window = $this_window;
        }
        if ( $time <= $window->{max} ) {
            return $prev_window;
        }
    }
    return $time . ' seconds';
}
