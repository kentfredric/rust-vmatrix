#!perl
use strict;
use warnings;

my (@crates) = (
    'advapi32-sys',               'aho-corasick',
    'cfg-if',                     'const_fn',
    'gcc',                        'kernel32-sys',
    'libc',                       'log',
    'memchr',                     'proc-macro-hack',
    'proc-macro2',                'rustversion',
    'time',                       'time-macros',
    'time-macros-impl',           'version_check',
    'winapi',                     'winapi-build',
    'winapi-i686-pc-windows-gnu', 'winapi-x86_64-pc-windows-gnu',
);

our $VERSION_BASE = "/home/kent/rust/vmatrix";
our $TEMPLATE     = "${VERSION_BASE}/index.html.tpl";
our $TARGET       = "${VERSION_BASE}/index.html";

sub crate_index_file {
    sprintf "%s/%s/%s", $VERSION_BASE, $_[0], "index.html";
}

sub gen_toc {
    my %crate_buckets;
    for my $crate (@crates) {
        if ( not -e crate_index_file($crate) ) {
            warn "No index.html for $crate, skipping\n";
            next;
        }
        my $bucket = substr $crate, 0, 1;
        push @{ $crate_buckets{$bucket} }, $crate;
    }

    my $inject = "";

    open my $fh, ">", \$inject or die "Can't open buffer for write";

    my $pad = " " x 4;
    for my $bucket ( sort keys %crate_buckets ) {
        $fh->printf(
            "$pad<h2 id=\"crate-%s\">%s*<a href=\"#crate-%s\">#</a></h2>\n",
            $bucket, $bucket, $bucket );
        $fh->printf("$pad<ul>\n");
        for my $crate ( sort @{ $crate_buckets{$bucket} } ) {
            $fh->printf( "$pad  <li><a href=\"./%s\">%s</a></li>\n",
                $crate, $crate );
        }
        $fh->printf("$pad</ul>\n");
    }
    close $fh or warn "Error closing buffer";
    return $inject;
}

my $code = do {
    open my $fh, "<", $TEMPLATE or die "Can't read $TEMPLATE, $!";
    local $/ = undef;
    scalar <$fh>;
};
$code =~ s{^\s*[<]!--\s*build\s+reports\s*--[>]\s*\n}{gen_toc}gmsex;
open my $fh, ">", $TARGET or die "Can't write $TEMPLATE, $!";
$fh->print($code);
close $fh or warn "error closing $TEMPLATE, $!";
