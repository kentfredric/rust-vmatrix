#!perl
use strict;
use warnings;

our $ROOT  = "/home/kent/rust/vmatrix";
our $CRATE = "gcc";
$CRATE = $ENV{CRATE} if exists $ENV{CRATE} and length $ENV{CRATE};
our $VINDEX      = "versions.txt";
our $INFO_PREFIX = "rustc-";

my %versions;
my %rustc_results;
my (@order);

for my $version ( get_versions("${ROOT}/${CRATE}/${VINDEX}") ) {
    next
      if exists $version->{message}
      and $version->{message}
      and $version->{message} eq 'NOEXIST';
    next
      if exists $version->{message}
      and $version->{message}
      and $version->{message} eq 'YANKED';
    my $v = $version->{version};
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
my $padline = ( " " x 10 ) . ( " " x scalar @rustc_order );
my (@vpad)  = map { $padline } 0 .. $vspace;
my $index   = 0;
for (@rustc_order) {
    my (@chars) = split //, $_;
    my $row_index = $vspace;
    for my $char ( reverse @chars ) {
        substr $vpad[$row_index], $index + 10, 1, $char;
        $row_index--;
    }
    $index++;
}
for (@vpad) {
    printf "%s\n", $_;
}

for my $version ( reverse @order ) {
    printf "%8s: ", $version;
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
