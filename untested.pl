#!perl
use strict;
use warnings;

use lib "/home/kent/rust/vcheck/lib/";
use lib "/home/kent/perl/kentnl/JSON-PrettyCompact/lib";
use resultdb;

my $rdb = resultdb->new();
for my $crate ( $rdb->crate_names ) {
    my $info = $rdb->crate_info($crate);
    next if -e $info->result_json_path();
    my (@versions) = grep { not $info->is_yanked($_) } ( @{ $info->versions } );
    next unless @versions;
    my (@deps) = ( keys %{ $info->dependencies } );
    my $mtime =
      -e $info->json_path
      ? sprintf "%d", 24 * 60 * -M $info->json_path
      : "?";
    printf "%d %d %s %s\n", scalar @versions, scalar @deps, $mtime, $crate;
}
