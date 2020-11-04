#!perl
use strict;
use warnings;

use lib "/home/kent/rust/vcheck/lib/";
use lib "/home/kent/perl/kentnl/JSON-PrettyCompact/lib";
use resultdb;

my $rdb = resultdb->new();

our $ROOT  = $rdb->root();
our $CRATE = "libc";
$CRATE = $ENV{CRATE} if exists $ENV{CRATE} and length $ENV{CRATE};
our $OUTFILE = "index.html";

my $crateinfo = $rdb->crate_info($CRATE);

my %versions;
my (@order);

for my $version ( @{ $crateinfo->versions } ) {
    next if $crateinfo->is_yanked($version);
    push @order, $version;
    $versions{$version} = {} unless exists $versions{$version};
}
my (@rustc_order) = sort { vsort( $a, $b ) } @{ $crateinfo->rustcs };

my $outbuf = "";

open my $fh, ">", \$outbuf or die "Can't write output buffer";
$fh->print(<<"EOF");
<html>
  <head>
    <title>Test results for crate:${CRATE} vs rustc</title>
    <style>
    * {
      font-family: monospace;
      font-size: 10px;
    }
    table {
      table-layout: fixed;
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
    th.rustversion, th.crateversionheading {
      writing-mode: sideways-lr;
      /* Sorry Chrome and Opera, you're on your fucking own
       */
      -webkit-writing-mode: vertical-lr;
      white-space: nowrap;
      padding: 1px;
    }
    th.crateversionheading {
      width: 15px;
    }
    th.rustversion, td.result {
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
$fh->print("<td class=\"corner\" colspan=\"2\"></td>\n");
$fh->printf(
    "<th class=\"rustversionheading\" colspan=\"%s\">Rust Version</td>\n",
    scalar @rustc_order );
$fh->print("</tr><tr>\n");
$fh->print("<td class=\"corner\" colspan=\"2\"></td>\n");

for (@rustc_order) {
    $fh->printf( "<th class=\"rustversion\">%s</th>\n", $_ );
}
$fh->print("</tr></thead>\n");
$fh->print("<tbody>\n");
my $is_first = 1;
for my $version ( reverse @order ) {
    $fh->print("<tr>\n");
    if ($is_first) {
        undef $is_first;
        $fh->printf(
"<th class=\"crateversionheading\" rowspan=\"%s\">Crate:%s version</th>",
            scalar @order,
            $CRATE
        );
    }
    $fh->printf( "<th class=\"crateversion\">%s</th>\n", $version );
    for my $rustc (@rustc_order) {
        my $result = $crateinfo->rustc_version_result( $rustc, $version );
        if ( not length $result ) {
            $fh->printf(
"<td class=\"result unknown\" title=\"unknown result for %s version %s on rust %s\">?</td>\n",
                $CRATE, $version, $rustc );
            next;
        }
        if ( 'pass' eq $result ) {
            $fh->printf(
"<td class=\"result pass\" title=\"pass for %s version %s on rust %s\">*</td>\n",
                $CRATE, $version, $rustc );
            next;
        }
        if ( 'fail' eq $result ) {
            $fh->printf(
"<td class=\"result fail\" title=\"fail for %s version %s on rust %s\">_</td>\n",
                $CRATE, $version, $rustc );
            next;
        }
    }
    $fh->print("</tr>\n");
}
$fh->print("</tbody></table></body></html>\n");
close $fh;
{
    open my $fh, ">", "${ROOT}/${CRATE}/${OUTFILE}"
      or die "Can't write $OUTFILE, $!";
    $fh->print($outbuf);
    close $fh;
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
