#!/usr/bin/env perl
use strict;
use warnings;

our $VERSION = '0.0';

use lib "/home/kent/rust/vcheck/lib/";
use cargo;

my $crate = shift @ARGV;

my $ROOT = "/home/kent/rust/vmatrix/";

die "No crate specified" if not defined $crate;

my (@versions) = @{ cargo::get_version_info($crate) };

my $CRATE_VERSIONS = "${ROOT}/$crate/versions.txt";

my $out = "";
open my $buf, ">", \$out or die "can't open writeable buffer";

for my $version ( reverse @versions ) {
    my (@parts);
    push @parts, $version->{num};
    if ( $version->{yanked} ) {
        push @parts, 'YANKED';
    }
    $buf->printf( "%s\n", join '|', @parts );
}
close $buf;

open my $fh, ">", $CRATE_VERSIONS or die "Can't write to $CRATE_VERSIONS, $!";
$fh->print($out);
close $fh or warn "error closing $CRATE_VERSIONS, $!";
