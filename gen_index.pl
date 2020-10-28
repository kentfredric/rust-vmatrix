#!perl
use strict;
use warnings;

our $VERSION_BASE = "/home/kent/rust/vmatrix";
our $TEMPLATE     = "${VERSION_BASE}/index.html.tpl";
our $TARGET       = "${VERSION_BASE}/index.html";

sub find_crates {
    opendir my $dfh, $VERSION_BASE or die "Can't opendir $VERSION_BASE";
    my (@crates);
    while ( my $ent = readdir $dfh ) {
        next if $ent eq '.';
        next if $ent eq '..';
        next unless -d "${VERSION_BASE}/${ent}";
        next unless -r "${VERSION_BASE}/${ent}/versions.txt";
        next unless -r "${VERSION_BASE}/${ent}/index.html";
        push @crates, $ent;
    }
    return @crates;
}

sub grade_crate {
    my ($crate) = @_;
    my $cratedir = "${VERSION_BASE}/${crate}";
    my ( $npass, $nfail ) = ( 0, 0 );
    opendir my $dfh, $cratedir or die "Can't opendir $cratedir";
    while ( my $ent = readdir $dfh ) {
        next if $ent eq '.';
        next if $ent eq '..';
        next unless $ent =~ m/\Arustc-\d+[.]\d+[.]\d+\z/;
        next if -d $ent;
        my $file = "$cratedir/$ent";
        open my $fh, "<", "$file" or die "Can't read $file, $!";
        while ( my $line = <$fh> ) {
            chomp $line;
            my ( $version, $grade, @extra ) = split /[|]/, $line;
            $npass++ if $grade eq 'pass';
            $nfail++ if $grade eq 'fail';
        }
    }
    return -1 if $npass > 1 and $nfail == 0;
    my $pct_fail = sprintf "%0.1f", $nfail / ( $npass + $nfail );
    return $pct_fail;
}

sub gen_toc {
    my %crate_buckets;
    for my $crate (find_crates) {
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
            $fh->printf( "$pad  <li><a href=\"./%s\">%s</a>", $crate, $crate );
            my $grade = grade_crate($crate);
            if ( $grade < 0 ) {
                $fh->print(
"<span class=\"grade goldstar\" title=\"No reported failures for any version on any rust\">&#x1F31F;</span>"
                );
            }
            else {
                $fh->printf(
"<span class=\"grade numeric_%d\" title=\"%d%% failures\"></span>",
                    $grade * 10,
                    $grade * 100
                );
            }
            $fh->print("</li>\n");
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
