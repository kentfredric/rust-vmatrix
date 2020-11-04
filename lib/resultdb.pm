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

sub rustc_order {
    qw(
      1.0.0  1.1.0  1.2.0  1.3.0  1.4.0  1.5.0  1.6.0  1.7.0  1.8.0  1.9.0
      1.10.0 1.11.0 1.12.1 1.13.0 1.14.0 1.15.1 1.16.0 1.17.0 1.18.0 1.19.0
      1.20.0 1.21.0 1.22.1 1.23.0 1.24.1 1.25.0 1.26.2 1.27.2 1.28.0 1.29.2
      1.30.1 1.31.1 1.32.0 1.33.0 1.34.2 1.35.0 1.36.0 1.37.0 1.38.0 1.39.0
      1.40.0 1.41.1 1.42.0 1.43.1 1.44.1 1.45.2 1.46.0 1.47.0
    );
}

sub root             { $_[0]->{root} }
sub crate_dir        { $_[0]->{root} . '/' . $_[1] }
sub crate_vjson_path { $_[0]->crate_dir( $_[1] ) . '/versions.json' }
sub crate_rjson_path { $_[0]->crate_dir( $_[1] ) . '/results.json' }

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

sub crate_write_rjson {
    my ( $self, $crate, $results ) = @_;
    my $crate_rjson = $self->crate_rjson_path($crate);
    my $encoder     = JSON::PrettyCompact->new( width => 69, indent => 2 );
    my $out         = $encoder->encode($results);
    open my $fh, ">:utf8", $crate_rjson
      or die "Can't write to $crate_rjson, $!";
    $fh->print($out);
    close $fh or warn "error closing $crate_rjson, $!";
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

sub crate_read_rjson {
    my ( $self, $crate ) = @_;
    my $jxs         = JSON::MaybeXS->new();
    my $crate_rjson = $self->crate_rjson_path($crate);
    return [] unless -e $crate_rjson;
    open my $fh, "<:utf8", $crate_rjson or die "Can't read $crate_rjson, $!";
    return $jxs->decode(
        do {
            local $/;
            scalar <$fh>;
        }
    );
}

sub crate_dependencies_from_json {
    my ( $self, $json ) = @_;
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

sub crate_dependencies {
    my ( $self, $crate ) = @_;
    return $self->crate_dependencies_from_json(
        $self->crate_read_vjson($crate) );
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
        push @rustcs, $1;
    }
    return \@rustcs;
}

sub crate_flat_rustc_results {
    my ( $self, $crate, $rustc ) = @_;
    die "crate missing" if not defined $crate or not length $crate;
    die "rustc missing" if not defined $rustc or not length $rustc;
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

sub crate_info {
    require resultdb::crateinfo;
    resultdb::crateinfo->new( rdb => $_[0], crate => $_[1] );
}

sub crate_write_flat_rustc_results {
    my ( $self, $crate, $rustc, $results ) = @_;
    my $file = $self->crate_dir($crate) . '/rustc-' . $rustc;
    open my $fh, ">", $file or die "Can't write $file, $!";
    for my $result ( @{$results} ) {
        $fh->printf( "%s|%s\n", $result->{version}, $result->{message} );
    }
    close $fh or warn "Error closing $file, $!\n";
}

sub _merge_rustc {
    my ( $self, $old, $new ) = @_;
    my (%results) = (
        ( map { $_ => "pass" } @{ $old->{rustc_pass} } ),
        ( map { $_ => "fail" } @{ $old->{rustc_fail} } ),
        ( map { $_ => "pass" } @{ $new->{rustc_pass} } ),
        ( map { $_ => "fail" } @{ $new->{rustc_fail} } ),
    );
    return {
        %{$old},
        %{$new},
        rustc_fail => [
            grep { exists $results{$_} and $results{$_} eq 'fail' }
              $self->rustc_order
        ],
        rustc_pass => [
            grep { exists $results{$_} and $results{$_} eq 'pass' }
              $self->rustc_order
        ],
    };

}

sub crate_merge_flat_rustc_results {
    my ( $self, $crate, $old_results, $new_rustc, $new_results, $json_data ) =
      @_;
    my $mock_new = [];
    for my $new_result ( @{$new_results} ) {
        my $rec = { crate => $crate, num => $new_result->[0] };
        if ( $new_result->[1] and $new_result->[1] eq 'pass' ) {
            $rec->{rustc_pass} = [$new_rustc];
        }
        if ( $new_result->[1] and $new_result->[1] eq 'fail' ) {
            $rec->{rustc_fail} = [$new_rustc];
        }
        push @{$mock_new}, $rec;
    }
    $self->crate_merge_rustc_results( $crate, $old_results, $mock_new,
        $json_data );
}

sub crate_merge_rustc_results {
    my ( $self, $crate, $old_results, $new_results, $json_data ) = @_;
    $old_results = [] if not defined $old_results;
    my (%old_data);
    for my $result ( @{$old_results} ) {
        $old_data{ $result->{num} } = $result;
    }
    my (%new_data);
    for my $result ( @{$new_results} ) {
        $new_data{ $result->{num} } = $result;
    }
    my (@out);
    my (@versions) = map { $_->{num} }
      grep { not exists $_->{yanked} or not $_->{yanked} } @{$json_data};

    for my $version (@versions) {
        if (    not exists $old_data{$version}
            and not exists $new_data{$version} )
        {
            push @out, { num => $version, };
            next;
        }
        if ( not exists $old_data{$version} and exists $new_data{$version} ) {
            push @out, $new_data{$version};
            next;
        }
        if ( exists $old_data{$version} and not exists $new_data{$version} ) {
            push @out, $old_data{$version};
            next;
        }
        push @out,
          $self->_merge_rustc( $old_data{$version}, $new_data{$version} );

    }
    return \@out;
}
1;
