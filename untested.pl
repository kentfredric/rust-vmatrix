#!perl
use strict;
use warnings;

use lib "/home/kent/rust/vcheck/lib/";
use lib "/home/kent/perl/kentnl/JSON-PrettyCompact/lib";
use resultdb;

my $rdb = resultdb->new();
for my $crate ( $rdb->crate_names ) {
    my $info = $rdb->crate_info($crate);
    next if @{ $info->rustcs };
    my (@versions) = ( @{ $info->versions } );
    next unless @versions;
    my (@deps) = ( keys %{ $info->dependencies } );
    printf "%d %d %s\n", scalar @versions, scalar @deps, $crate;
}
