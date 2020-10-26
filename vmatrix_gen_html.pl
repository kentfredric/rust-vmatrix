#!perl
use strict;
use warnings;

our $ROOT        = "/home/kent/rust/vmatrix";
our $CRATE       = "gcc";
our $VINDEX      = "versions.txt";
our $OUTFILE     = "index.html";
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

open my $fh, ">", "${ROOT}/${CRATE}/${OUTFILE}"
  or die "Can't write $OUTFILE, $!";

$fh->print(<<"EOF");
<html>
  <head>
    <title>Test results for crate:${CRATE} vs rustc</title>
    <style>
    * {
      font-family: monospace;
      font-size: 10px;
    }
    td.fail {
      background-color: #FF0000;
      color: #FFFFFF;
    }
    td.pass {
      color: #000000;
      background-color: #00FF00;
      font-weight: 900;
    }
    td.rustversion {
      writing-mode: sideways-lr;
      text-orientation: sideways;
    }
    td.rustversion, td.result {
      width: 15px;
      height: 15px;
    }
    td.result {
      text-align: center;
    }
    </style>
  </head>
  <body>
EOF
$fh->print("<table>\n");
$fh->print("<thead><tr>\n");
$fh->print("<td class=\"corner\"></td>\n");
for (@rustc_order) {
    $fh->printf( "<td class=\"rustversion\">%s</td>\n", $_ );
}
$fh->print("</tr></thead>\n");
$fh->print("<tbody>\n");
for my $version ( reverse @order ) {
    $fh->print("<tr>\n");
    $fh->printf( "<td class=\"crateversion\">%s</td>\n", $version );
    for my $rustc (@rustc_order) {
        if ( not exists $rustc_results{$rustc}{$version} ) {
            $fh->printf(
"<td class=\"result unknown\" title=\"unknown result for %s version %s on rust %s\">?</td>\n",
                $CRATE, $version, $rustc );
            next;
        }
        if ( $rustc_results{$rustc}{$version} eq 'pass' ) {
            $fh->printf(
"<td class=\"result pass\" title=\"pass for %s version %s on rust %s\">*</td>\n",
                $CRATE, $version, $rustc );
            next;
        }
        if ( $rustc_results{$rustc}{$version} eq 'fail' ) {
            $fh->printf(
"<td class=\"result fail\" title=\"fail for %s version %s on rust %s\">_</td>\n",
                $CRATE, $version, $rustc );
            next;
        }
    }
    $fh->print("</tr>\n");
}
$fh->print("</tbody></table></body></html>\n");

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
