use 5.006;    # our
use strict;
use warnings;

package cargo;

our $VERSION = '0.001000';

# ABSTRACT: Interface for cargo stuff

# AUTHORITY

our $CRATES_BASE_URL = "https://crates.io/api/v1/crates";
our $USER_AGENT      = do {
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

sub get_version_info {
    my ($crate) = @_;

    my $response =
      $HTTP_TINY->get( $CRATES_BASE_URL . '/' . $crate . '/versions' );
    die "Failed to get info for crate $crate: " . pp($response)
      unless $response->{success};
    my $blob = $JXS->decode( $response->{content} );
    [ @{ $blob->{versions} } ];

}

1;

