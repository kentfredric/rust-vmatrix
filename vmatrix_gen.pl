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
our $INFO_PREFIX = "rustc-";

my %versions;
my %rustc_results;
my (@order);
my $crate_vspace = 0;
for my $version ( @{ $rdb->crate_read_vjson($CRATE) } ) {
    next if exists $version->{yanked} and $version->{yanked};
    my $v = $version->{num};
    if ( length $v > $crate_vspace ) {
        $crate_vspace = length $v;
    }
    push @order, $v;
    $versions{$v} = {} unless exists $versions{$v};
}
{
    opendir my $dfh, "${ROOT}/${CRATE}"
      or die "Can't opendir ${ROOT}/${CRATE}, $!";
    while ( my $ent = readdir $dfh ) {
        next if $ent eq '.';
        next if $ent eq '..';
        next unless $ent =~ /\A\Q$INFO_PREFIX\E(\d+.\d+.\d+)\z/;
        my $rustc = $1;
        my (@results) = get_versions("${ROOT}/${CRATE}/$ent");
        $rustc_results{$rustc} = {} unless exists $rustc_results{$rustc};
        for my $version (@results) {
            my $v = $version->{version};
            next unless exists $versions{$v};
            $rustc_results{$rustc}{$v} = $version->{message};
        }
    }
}
my (@rustc_order) = sort { vsort( $a, $b ) } keys %rustc_results;

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
        if ( not exists $rustc_results{$rustc}{$version} ) {
            print "?";
            next;
        }
        if ( $rustc_results{$rustc}{$version} eq 'pass' ) {
            print "\e[32;1m*\e[0m";
            next;
        }
        if ( $rustc_results{$rustc}{$version} eq 'fail' ) {
            print "\e[31m_\e[0m";
            next;
        }
    }
    print "\n";
}

sub get_versions {
    my ($path) = @_;
    open my $fh, "<", $path or die "can't read $path";
    my @v;
    while ( my $line = <$fh> ) {
        chomp $line;
        my ( $version, $message, @rest ) = split /[|]/, $line;
        my $rec = { version => $version };
        if ( defined $message and length $message ) {
            $rec->{message} = $message;
        }
        if (@rest) {
            $rec->{extras} = \@rest;
        }
        push @v, $rec;
    }
    return @v;
}

sub vsplit {
    split /[.]/;
}

sub vsort {
    my ( $lhs, $rhs ) = @_;
    my (@lhs_parts) = split /[.]/, $lhs;
    my (@rhs_parts) = split /[.]/, $rhs;

    $lhs_parts[0] <=> $rhs_parts[0]
      or $lhs_parts[1] <=> $rhs_parts[1]
      or $lhs_parts[2] <=> $rhs_parts[2];
}
