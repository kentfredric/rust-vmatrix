#!perl
use strict;
use warnings;

use lib "/home/kent/rust/vcheck/lib/";
use lib "/home/kent/perl/kentnl/JSON-PrettyCompact/lib";
use resultdb;

my $rdb = resultdb->new();
for my $crate ( $rdb->crate_names ) {
    my $info = $rdb->crate_info($crate);
    next unless -e $info->result_json_path();
    next unless $info->has_results();
    my (@untested) = @{ $info->untested_combos };
    if (@untested) {
        printf "\e[33m%s\e[0m\n", $crate;
    }
    for my $untested (@untested) {
        printf " %s v=%s rustc=%s\n", $crate, $untested->{num},
          $untested->{rustc};
    }
}
