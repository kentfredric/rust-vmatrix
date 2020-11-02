#!perl
use strict;
use warnings;

use lib "/home/kent/rust/vcheck/lib/";
use lib "/home/kent/perl/kentnl/JSON-PrettyCompact/lib";
use resultdb;

my $rdb = resultdb->new();
for my $crate ( $rdb->crate_names ) {
    next if ( @{ $rdb->crate_flat_rustcs($crate) } );
    my (@versions) = ( @{ $rdb->crate_read_vjson($crate) } );
    next unless @versions;
    printf "%d %s\n", scalar @versions, $crate;
}
