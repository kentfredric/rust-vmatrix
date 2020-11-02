#!/usr/bin/env perl
use strict;
use warnings;

our $VERSION = '0.0';

use lib "/home/kent/rust/vcheck/lib/";
use lib "/home/kent/perl/kentnl/JSON-PrettyCompact/lib";
use cargo;
use resultdb;

my $rdb = resultdb->new();

for
  my $crate ( sort { ( rand() * 100 ) <=> ( rand() * 100 ) } $rdb->crate_names )
{
    next if -e $rdb->crate_vjson_path($crate);
    my $old_versions = $rdb->crate_read_vjson($crate);
    my (@versions) = @{ cargo::update_version_info( $crate, $old_versions ) };
    $rdb->crate_write_vfile( $crate, [ reverse @versions ] );
    $rdb->crate_write_vjson( $crate, [ reverse @versions ] );
}
