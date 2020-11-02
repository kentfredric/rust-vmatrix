#!/usr/bin/env perl
use strict;
use warnings;

our $VERSION = '0.0';

use lib "/home/kent/rust/vcheck/lib/";
use lib "/home/kent/perl/kentnl/JSON-PrettyCompact/lib";
use resultdb;

my $rdb = resultdb->new();

for my $key ( sort keys %{ $rdb->all_crate_dependencies() } ) {
    my $path = $rdb->root . '/' . $key;
    if ( !-d $path ) {
        warn "Missing dep $key\n";
        system( 'mkdir', '-p', $path );
    }
}
