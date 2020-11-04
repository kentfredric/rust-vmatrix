#!/usr/bin/env perl
use strict;
use warnings;

our $VERSION = '0.0';

use lib "/home/kent/rust/vcheck/lib/";
use lib "/home/kent/perl/kentnl/JSON-PrettyCompact/lib";
use resultdb;

my $rdb = resultdb->new();
for my $crate ( $rdb->crate_names ) {
    my (@rustcs) = @{ $rdb->crate_flat_rustcs($crate) };
    next unless @rustcs;
    printf "%s : Reading $crate\n", scalar gmtime;
    my $results   = [];
    my $json_data = $rdb->crate_read_vjson($crate);
    for my $rustc (@rustcs) {
        my $flat_results = $rdb->crate_flat_rustc_results( $crate, $rustc );
        $results =
          $rdb->crate_merge_flat_rustc_results( $crate, $results, $rustc,
            $flat_results, $json_data );
    }
    printf "%s : Writing $crate\n", scalar gmtime;
    $rdb->crate_write_rjson( $crate, $results );
}
