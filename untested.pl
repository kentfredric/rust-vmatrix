#!perl
use strict;
use warnings;

use lib "/home/kent/rust/vcheck/lib/";
use lib "/home/kent/perl/kentnl/JSON-PrettyCompact/lib";
use resultdb;

my $rdb = resultdb->new();
my (@result_rows) = @_;

for my $crate ( $rdb->crate_names ) {
    my $info = $rdb->crate_info($crate);
    next if -e $info->result_json_path();
    my (@versions) = grep { not $info->is_yanked($_) } ( @{ $info->versions } );
    next unless @versions;
    my (@deps) = ( keys %{ $info->dependencies } );
    if ( not $ENV{PICK} ) {
        my $mtime =
          -e $info->json_path
          ? sprintf "%d", 24 * 60 * -M $info->json_path
          : "?";
        printf "%d %d %s %s\n", scalar @versions, scalar @deps, $mtime, $crate;
    }
    else {
        push @result_rows,
          [ scalar @versions, scalar @deps, $info->json_path, $crate ];
    }
}
if ( $ENV{PICK} ) {
    my $pick_many = $ENV{PICK_NUMBER} || 5;
    my $sort_fn   = sub { };

    if ( $ENV{PICK} eq 'small' ) {
        $sort_fn = sub {
                 $a->[0] <=> $b->[0]
              or $a->[1] <=> $b->[1]
              or ( -M $a->[2] ) <=> ( -M $b->[2] )
              or $a->[3] cmp $b->[3];
        };
    }
    if ( $ENV{PICK} eq 'smalldeps' ) {
        $sort_fn = sub {
                 $a->[1] <=> $b->[1]
              or $a->[0] <=> $b->[0]
              or ( -M $a->[2] ) <=> ( -M $b->[2] )
              or $a->[3] cmp $b->[3];
        };
    }
    if ( $ENV{PICK} eq 'bigsmalldeps' ) {
        $sort_fn = sub {
                 $a->[1] <=> $b->[1]
              or $b->[0] <=> $a->[0]
              or ( -M $a->[2] ) <=> ( -M $b->[2] )
              or $a->[3] cmp $b->[3];
        };
    }

    if ( $ENV{PICK} eq 'big' ) {
        $sort_fn = sub {
                 $b->[0] <=> $a->[0]
              or $b->[1] <=> $a->[1]
              or ( -M $a->[2] ) <=> ( -M $b->[2] )
              or $a->[3] cmp $b->[3];
        };
    }
    if ( $ENV{PICK} eq 'bigdeps' ) {
        $sort_fn = sub {
                 $b->[1] <=> $a->[1]
              or $b->[0] <=> $a->[0]
              or ( -M $a->[2] ) <=> ( -M $b->[2] )
              or $a->[3] cmp $b->[3];
        };
    }
    if ( $ENV{PICK} eq 'smallbigdeps' ) {
        $sort_fn = sub {
                 $a->[0] <=> $b->[0]
              or $b->[1] <=> $a->[1]
              or ( -M $a->[2] ) <=> ( -M $b->[2] )
              or $a->[3] cmp $b->[3];
        };
    }
    @result_rows = sort $sort_fn @result_rows;
    $pick_many   = scalar @result_rows if $pick_many > scalar @result_rows;
    for my $wanted ( splice( @result_rows, 0, $pick_many ) ) {
        printf "%d %d %d %s\n", $wanted->[0], $wanted->[1],
          24 * 60 * -M $wanted->[2], $wanted->[3];

    }
}
