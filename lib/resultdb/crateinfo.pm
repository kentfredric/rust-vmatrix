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

sub json_path {
    $_[0]->{rdb}->crate_vjson_path( $_[0]->{crate} );
}

sub result_json {
    if ( not exists $_[0]->{result_json} ) {
        $_[0]->{result_json} = $_[0]->{rdb}->crate_read_rjson( $_[0]->{crate} );
    }
    return $_[0]->{result_json};
}

sub result_json_path {
    $_[0]->{rdb}->crate_rjson_path( $_[0]->{crate} );
}

sub _as_result_hash {
    my ( $self, $record ) = @_;
    return {
        ( map { ( $_ => "pass" ) } @{ $record->{rustc_pass} || [] }, ),
        ( map { ( $_ => "fail" ) } @{ $record->{rustc_fail} || [] }, ),
    };
}

sub has_results {
    my ($self) = @_;
    for my $ent ( @{ $self->result_json } ) {
        return 1 if exists $ent->{rustc_pass} and @{ $ent->{rustc_pass} };
        return 1 if exists $ent->{rustc_fail} and @{ $ent->{rustc_fail} };
    }
    return;
}

sub srv_min_version {
    my ( $self, $version ) = @_;
    for my $result ( @{ $self->result_json } ) {
        next unless $result->{num} eq $version;
        my (@pass) = @{ $result->{rustc_pass} || [] };
        return $pass[0];
    }
    return;
}

sub srv_max_version {
    my ( $self, $version ) = @_;
    for my $result ( @{ $self->result_json } ) {
        next unless $result->{num} eq $version;
        my (@pass) = @{ $result->{rustc_pass} || [] };
        return $pass[-1];
    }
    return;
}

sub all_results {
    my ($self) = @_;
    my %results =
      map { $_->{num}, $self->_as_result_hash($_) } @{ $self->result_json };
    my $versions = $self->json;

    my $rustc_results = sub {
        my ( $version, $rustc ) = @_;
        return () if not exists $results{$version};
        return () if not exists $results{$version}{$rustc};
        return {
            num    => $version,
            rustc  => $rustc,
            result => $results{$version}{$rustc},
        };
    };
    my $version_results = sub {
        my ($version) = @_;
        return () if not exists $results{$version};
        return
          map { $rustc_results->( $version, $_ ) } $self->{rdb}->rustc_order();
    };
    [ map { $version_results->( $_->{num} ) } @{$versions} ];
}

sub untested_combos {
    my ($self) = @_;
    my (%results);
    for my $result ( @{ $self->result_json } ) {
        $results{ $result->{num} } = $self->_as_result_hash($result);
    }
    my $versions      = $self->json;
    my $rustc_results = sub {
        my ( $version, $rustc ) = @_;
        return { num => $version, rustc => $rustc }
          if not exists $results{$version}
          or not exists $results{$version}{$rustc};
        return ();
    };
    my $version_results = sub {
        my ($version) = @_;
        return () if $self->is_yanked($version);
        return
          map { $rustc_results->( $version, $_ ) } $self->{rdb}->rustc_order();
    };
    [ map { $version_results->( $_->{num} ) } @{$versions} ];
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
    if ( not exists $_[0]->{rustc_version_results} ) {
        $self->rustc_version_result($rustc);
    }
    if ( not exists $_[0]->{rustc_results}->{$rustc} ) {
        $self->{rustc_results}->{$rustc} = [
            map { [ $_, $_[0]->{rustc_version_results}->{$rustc}->{$_} ] }
              grep {
                      exists $_[0]->{rustc_version_results}->{$rustc}
                  and exists $_[0]->{rustc_version_results}->{$rustc}->{$_}
              } @{ $self->versions }
        ];

    }
    return $_[0]->{rustc_results}->{$rustc};
}

sub rustc_version_result {
    my ( $self, $rustc, $version ) = @_;
    if ( not exists $self->{rustc_version_results} ) {
        for my $result ( @{ $self->result_json } ) {
            my $ver = $result->{num};
            for my $pass_rust ( @{ $result->{rustc_pass} || [] } ) {
                $self->{rustc_version_results}->{$pass_rust}->{$ver} =
                  'pass';
            }
            for my $fail_rust ( @{ $result->{rustc_fail} || [] } ) {
                $self->{rustc_version_results}->{$fail_rust}->{$ver} =
                  'fail';
            }
        }
    }
    return "" if not defined $version;
    return ""
      if not exists $self->{rustc_version_results}->{$rustc}->{$version};
    return $self->{rustc_version_results}->{$rustc}->{$version};
}

1;
