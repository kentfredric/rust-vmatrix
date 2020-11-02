use 5.006;    # our
use strict;
use warnings;

package resultdb;

our $VERSION = '0.001000';

use JSON::PrettyCompact;
use JSON::MaybeXS;
use Data::Dump qw(pp);

# ABSTRACT: Map result trees

# AUTHORITY

sub new {
    my ( $class, @args ) = @_;
    my $self = bless {
        root => "/home/kent/rust/vmatrix/",
        ref $args[0] ? %{ $args[0] } : @args
    }, $class;
}

sub root             { $_[0]->{root} }
sub crate_dir        { $_[0]->{root} . '/' . $_[1] }
sub crate_vjson_path { $_[0]->crate_dir( $_[1] ) . '/versions.json' }

sub crate_names {
    my $cd = $_[0]->root;
    opendir my $dfh, $cd or die "cant opendir $cd, $!";
    my (@out);
    while ( my $ent = readdir $dfh ) {
        next if $ent =~ /^[.]/;
        next unless -d "$cd/$ent";
        push @out, $ent;
    }
    return @out;
}

sub crate_write_vjson {
    my ( $self, $crate, $versions ) = @_;
    my $crate_vjson = $self->crate_vjson_path($crate);
    open my $fh, ">:utf8", $crate_vjson or die "Cant write to $crate_vjson, $!";
    my $encoder = JSON::PrettyCompact->new( width => 69, indent => 2 );
    $fh->print( $encoder->encode($versions) );
    close $fh or warn "error closing $crate_vjson, $!";
}

sub crate_read_vjson {
    my ( $self, $crate ) = @_;
    my $jxs         = JSON::MaybeXS->new();
    my $crate_vjson = $self->crate_vjson_path($crate);
    return [] unless -e $crate_vjson;
    open my $fh, "<:utf8", $crate_vjson or die "Can't read $crate_vjson, $!";
    return $jxs->decode(
        do {
            local $/;
            scalar <$fh>;
        }
    );

}

sub crate_dependencies {
    my ( $self, $crate ) = @_;
    my $json = $self->crate_read_vjson($crate);
    return {} unless @{$json};
    my %dephash;
    for my $version ( @{$json} ) {
        next unless exists $version->{dependencies};
        next unless 'ARRAY' eq ref $version->{dependencies};
        next unless @{ $version->{dependencies} };
        for my $dep ( @{ $version->{dependencies} } ) {
            $dephash{ $dep->{crate_id} } = 1;
        }
    }
    return \%dephash;
}

sub all_crate_dependencies {
    my ($self) = @_;
    my %dephash;
    for my $crate ( $self->crate_names ) {
        %dephash = ( %dephash, %{ $self->crate_dependencies($crate) } );
    }
    return \%dephash;
}

sub crate_flat_rustcs {
    my ( $self, $crate ) = @_;
    my ($crate_dir) = $self->crate_dir($crate);
    my (@rustcs);
    opendir my $dfh, $crate_dir or die "Can't opendir $crate_dir, $!";
    while ( my $ent = readdir $dfh ) {
        next if $ent     =~ /\A[.]/;
        next unless $ent =~ /\Arustc-(.*)\z/;
        push @rustcs, $ent;
    }
    return \@rustcs;
}

sub crate_flat_rustc_results {
    my ( $self, $crate, $rustc ) = @_;
    my $rustc_file = $self->crate_dir($crate) . '/rustc-' . $rustc;
    if ( not -e $rustc_file ) {
        return [];
    }
    open my $fh, "<", $rustc_file or die "Can't read $rustc_file, $!";
    my (@recs);
    while ( my $line = <$fh> ) {
        chomp $line;
        push @recs, [ split /[|]/, $line ];
    }
    return \@recs;
}
1;

