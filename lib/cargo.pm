use 5.006;    # our
use strict;
use warnings;

package cargo;

our $VERSION = '0.001000';

# ABSTRACT: Interface for cargo stuff

# AUTHORITY

our $CRATES_BASE_URL = "https://crates.io";

our $USER_AGENT = do {
    my $script = $0;
    $script =~ s/\A(^.*\/)//;
    sprintf "%s/%s(%s)", $script, __PACKAGE__ . ":" . $VERSION,
      join q[,],
      ( 'private use', 'u=Kent Fredric', 'complaints=kentnl@gentoo.org' );
};
our $HTTP_TINY = do {
    require HTTP::Tiny;
    HTTP::Tiny->new( agent => $USER_AGENT, timeout => 15 );
};
our $JXS = do {
    require JSON::MaybeXS;
    JSON::MaybeXS->new();
};

use Data::Dump qw(pp);

sub api_fetch {
    my ( $url, $fatal ) = @_;
    my $response = $HTTP_TINY->get( $CRATES_BASE_URL . $url );
    if ( not $response->{success} ) {
        if ($fatal) {
            die "Could not fetch $url: " . pp($response);
        }
        else {
            warn "Could not fetch $url: " . pp($response);
            return;
        }
    }
    return $JXS->decode( $response->{content} );
}

sub get_version_info {
    my ($crate) = @_;
    update_version_info( $crate, [] );
}

sub update_version_info {
    my ( $crate, $old_info ) = @_;
    my %url_cache;
    for my $old_rec ( @{ $old_info || [] } ) {
        if (    exists $old_rec->{authors}
            and exists $old_rec->{links}->{authors} )
        {
            $url_cache{ $old_rec->{links}->{authors} } = $old_rec->{authors};
        }
        if (    exists $old_rec->{dependencies}
            and exists $old_rec->{links}->{dependencies} )
        {
            $url_cache{ $old_rec->{links}->{dependencies} } =
              $old_rec->{dependencies};
        }
    }
    warn "Getting crate info for $crate...\n";
    my $blob = api_fetch( '/api/v1/crates/' . $crate . '/versions', 1 );
    for my $version ( @{ $blob->{versions} } ) {
        if ( exists $version->{links} and exists $version->{links}->{authors} )
        {
            if ( exists $url_cache{ $version->{links}->{authors} } ) {
                $version->{authors} =
                  $url_cache{ $version->{links}->{authors} };
            }
            else {
                warn "Getting authors for $crate $version->{num}...\n";
                if ( my $ret = api_fetch( $version->{links}->{authors}, 0 ) ) {
                    $version->{authors} = $ret;
                }
            }
        }
        if (    exists $version->{links}
            and exists $version->{links}->{dependencies} )
        {
            if ( exists $url_cache{ $version->{links}->{dependencies} } ) {
                $version->{dependencies} =
                  $url_cache{ $version->{links}->{dependencies} };
            }
            else {
                warn "Getting dependencies for $crate $version->{num}...\n";
                if ( my $ret =
                    api_fetch( $version->{links}->{dependencies}, 0 ) )
                {
                    $version->{dependencies} = $ret->{dependencies};
                }
            }
        }
    }
    [ @{ $blob->{versions} } ];
}

1;

