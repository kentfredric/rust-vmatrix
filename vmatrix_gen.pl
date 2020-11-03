#!perl
use strict;
use warnings;

use lib "/home/kent/rust/vcheck/lib/";
use lib "/home/kent/perl/kentnl/JSON-PrettyCompact/lib";
use resultdb;

my $rdb = resultdb->new();

our $ROOT  = $rdb->root();
our $CRATE = "gcc";
$CRATE = $ENV{CRATE} if exists $ENV{CRATE} and length $ENV{CRATE};

my $crateinfo = $rdb->crate_info($CRATE);

my %versions;
my %rustc_results;
my (@order);
my $crate_vspace = 0;
for my $version ( @{ $crateinfo->versions } ) {
    next if $crateinfo->is_yanked($version);
    if ( length $version > $crate_vspace ) {
        $crate_vspace = length $version;
    }
    push @order, $version;
    $versions{$version} = {} unless exists $versions{$version};
}

my (@rustc_order) = sort { vsort( $a, $b ) } @{ $crateinfo->rustcs };

# Create a rotated heading, lmao
my $vspace = 0;
for (@rustc_order) {
    $vspace = length $_ if $vspace < length $_;
}

# Each line needs to be as long as the # of rust versions
my $padline = " " x scalar @rustc_order;

# And we need as many of them as the longest rust version has chars
my (@vpad) = map { $padline } 0 .. $vspace;

# The output *column* we're currently writing to
# relative to the rust-version block
my $index = 0;
for (@rustc_order) {
    my (@chars) = split //, $_;

    # start writing in the *last* row
    my $row_index = $#vpad;
    for my $char ( reverse @chars ) {

        # iteratively write upwards
        # writing into the column that correlates with our current version
        substr $vpad[$row_index], $index, 1, $char;
        $row_index--;
    }
    $index++;
}
for (@vpad) {

    # write the block, but with aligning indent
    printf " %s  %s\n", ( " " x $crate_vspace ), $_;
}

for my $version ( reverse @order ) {
    printf "%*s: ", $crate_vspace + 1, $version;
    for my $rustc (@rustc_order) {
        my $result = $crateinfo->rustc_version_result( $rustc, $version );
        if ( not length $result ) {
            print "?";
            next;
        }
        if ( 'pass' eq $result ) {
            print "\e[32;1m*\e[0m";
            next;
        }
        if ( 'fail' eq $result ) {
            print "\e[31m_\e[0m";
            next;
        }
    }
    print "\n";
}

sub vsort {
    my ( $lhs, $rhs ) = @_;
    my (@lhs_parts) = split /[.]/, $lhs;
    my (@rhs_parts) = split /[.]/, $rhs;

    $lhs_parts[0] <=> $rhs_parts[0]
      or $lhs_parts[1] <=> $rhs_parts[1]
      or $lhs_parts[2] <=> $rhs_parts[2];
}
