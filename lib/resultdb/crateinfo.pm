use 5.006;    # our
use strict;
use warnings;

package resultdb::crateinfo;

our $VERSION = '0.001000';

# ABSTRACT: Version metadata about a crate

# AUTHORITY

sub new {
    my ( $class, @args ) = @_;
    my $self = bless { ref $args[0] ? %{ $args[0] } : @args, }, $class;
    die "No rdb"   unless exists $self->{rdb};
    die "No crate" unless exists $self->{crate};
    $self;
}

sub versions {
    if ( not exists $_[0]->{versions} ) {
        $_[0]->{versions}     = [];
        $_[0]->{version_data} = {};
        for my $version ( @{ $_[0]->json } ) {
            push @{ $_[0]->{versions} }, $version->{num};
            $_[0]->{version_data}->{ $version->{num} } = $version;
        }
    }
    return $_[0]->{versions};
}

sub is_yanked {
    my ( $self, $version ) = @_;
    die "No version query" if not defined $version;
    $self->versions        if not exists $_[0]->{version_data};
    if ( exists $_[0]->{version_data}->{$version}->{yanked}
        and $_[0]->{version_data}->{$version}->{yanked} )
    {
        return 1;
    }
    return 0;
}

sub dependencies {
    if ( not exists $_[0]->{dependencies} ) {
        $_[0]->{dependencies} =
          $_[0]->{rdb}->crate_dependencies_from_json( $_[0]->json );
    }
    return $_[0]->{dependencies};
}

sub json {
    if ( not exists $_[0]->{json} ) {
        $_[0]->{json} = $_[0]->{rdb}->crate_read_vjson( $_[0]->{crate} );
    }
    return $_[0]->{json};
}

sub result_json {
    if ( not exists $_[0]->{result_json} ) {
        $_[0]->{result_json} = $_[0]->{rdb}->crate_read_rjson( $_[0]->{crate} );
    }
    return $_[0]->{result_json};
}

sub rustcs {
    if ( not exists $_[0]->{rustcs} ) {
        my (%rustcs);
        for my $version ( @{ $_[0]->result_json } ) {
            for my $rustc ( @{ $version->{rustc_fail} },
                @{ $version->{rustc_pass} } )
            {
                $rustcs{$rustc} = 1;
            }
        }
        $_[0]->{rustcs} =
          [ grep { exists $rustcs{$_} } $_[0]->{rdb}->rustc_order ];
    }
    return $_[0]->{rustcs};
}

sub rustc_results {
    my ( $self, $rustc ) = @_;
    die "rustc not passed" unless defined $rustc;
    if ( not exists $_[0]->{rustc_results}->{$rustc} ) {
        $_[0]->{rustc_results}->{$rustc} =
          $self->{rdb}->crate_flat_rustc_results( $self->{crate}, $rustc );
    }
    return $_[0]->{rustc_results}->{$rustc};
}

sub rustc_version_result {
    my ( $self, $rustc, $version ) = @_;
    if ( not exists $self->{rustc_version_results}->{$rustc} ) {
        for my $info ( @{ $self->rustc_results($rustc) } ) {
            if ( 1 < scalar @{$info} ) {
                $self->{rustc_version_results}->{$rustc}->{ $info->[0] } =
                  $info->[1];
            }
        }
    }
    return ""
      if not exists $self->{rustc_version_results}->{$rustc}->{$version};
    return $self->{rustc_version_results}->{$rustc}->{$version};
}

1;
