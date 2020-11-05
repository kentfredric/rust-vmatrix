#!perl
use strict;
use warnings;

use lib "/home/kent/rust/vcheck/lib/";
use lib "/home/kent/perl/kentnl/JSON-PrettyCompact/lib";
use resultdb;

my $CRATE = 'time';
$CRATE = $ENV{CRATE} if $ENV{CRATE};

my $rdb = resultdb->new();

# build dependency graph
my (@g) = get_events($CRATE);

my %events_traced;

sub get_events {
    my ($crate) = @_;
    if ( exists $events_traced{$crate} ) {
        return ();
    }
    $events_traced{$crate} = 1;

    my $crateinfo = $rdb->crate_info($crate);

    my @events;
    for my $version ( @{ $crateinfo->json } ) {
        my $cinfo;
        my $uinfo;
        my (@deps);
        if ( exists $version->{dependencies}
            and @{ $version->{dependencies} || [] } )
        {
            @deps = map {
                {
                    crate    => $_->{crate_id},
                    requires => $_->{req},
                    target   => $_->{target},
                    kind     => $_->{kind},
                    optional => $_->{optional}
                }
            } grep { $_->{kind} eq 'normal' and not $_->{optional} }
              @{ $version->{dependencies} };
        }
        for my $dep (@deps) {
            push @events, get_events( $dep->{crate} );
        }
        if ( exists $version->{created_at} and $version->{created_at} ) {
            $cinfo = {
                crate     => $version->{crate},
                action    => "create",
                timestamp => $version->{created_at},
                version   => $version->{num},
            };
            if (@deps) {
                $cinfo->{deps} = \@deps;
            }
        }
        if ( exists $version->{updated_at} and $version->{updated_at} ) {
            $uinfo = {
                crate     => $version->{crate},
                action    => "update",
                timestamp => $version->{created_at},
                version   => $version->{num},
            };

        }

        if ( exists $version->{audit_actions}
            and @{ $version->{audit_actions} || [] } )
        {
            for my $action ( @{ $version->{audit_actions} } ) {
                if ( $action->{action} eq 'publish' ) {
                    push @events,
                      {
                        crate     => $version->{crate},
                        version   => $version->{num},
                        action    => $action->{action},
                        timestamp => $action->{time},
                        ( @deps ? ( deps => \@deps ) : () ),
                      };
                    if ( defined $cinfo
                        and $cinfo->{timestamp} eq $action->{time} )
                    {
                        undef $cinfo;
                    }
                    if ( defined $uinfo
                        and $uinfo->{timestamp} eq $action->{time} )
                    {
                        undef $uinfo;
                    }
                    next;
                }
                if ( $action->{action} eq 'yank' ) {
                    push @events,
                      {
                        crate     => $version->{crate},
                        version   => $version->{num},
                        action    => $action->{action},
                        timestamp => $action->{time},
                      };
                    if ( defined $cinfo
                        and $cinfo->{timestamp} eq $action->{time} )
                    {
                        undef $cinfo;
                    }
                    if ( defined $uinfo
                        and $uinfo->{timestamp} eq $action->{time} )
                    {
                        undef $uinfo;
                    }
                    next;
                }
                if ( $action->{action} eq 'unyank' ) {
                    push @events,
                      {
                        crate     => $version->{crate},
                        version   => $version->{num},
                        action    => $action->{action},
                        timestamp => $action->{time},
                      };
                    if ( defined $cinfo
                        and $cinfo->{timestamp} eq $action->{time} )
                    {
                        undef $cinfo;
                    }
                    if ( defined $uinfo
                        and $uinfo->{timestamp} eq $action->{time} )
                    {
                        undef $uinfo;
                    }
                    next;
                }

                die "Unhandled action: " . $action->{action};
            }
        }
        if ( defined $cinfo and defined $uinfo ) {
            if ( $cinfo->{timestamp} = $uinfo->{timestamp} ) {
                undef $uinfo;
            }
        }

        push @events, $cinfo if defined $cinfo;
        push @events, $uinfo if defined $uinfo;
    }
    return @events;
}

sub bundle_events {
    my (@in) = @_;
    my (@out);
    my $bundle_last;
    my (@bundle);
    for my $event (@in) {
        my $sevent = substr $event->{timestamp}, 0, 4;
        if ( not defined $bundle_last ) {
            push @bundle, $event;
            $bundle_last = $sevent;
            next;
        }
        if ( $bundle_last ne $sevent ) {
            push @out, [@bundle] if @bundle;
            @bundle      = ();
            $bundle_last = $sevent;
            next;
        }
        push @bundle, $event;
    }
    if (@bundle) {
        push @out, [@bundle];
    }
    return @out;
}

my %repressed;
my %seen_deps = ( $CRATE, 1 );

for my $bundle (
    bundle_events( sort { $a->{timestamp} cmp $b->{timestamp} } @g ) )
{
    print $bundle->[0]->{timestamp};
    print "\n";
    my %rails    = ( $CRATE, 0 );
    my $rail_num = 1;
    my (@queue)  = sort { $a->{timestamp} cmp $b->{timestamp} } @{$bundle};
  event: while (@queue) {
        my $event = shift @queue;
        my $crate = $event->{crate};
        if ( not exists $seen_deps{$crate} ) {
            $repressed{$crate} = $event;
            next;
        }
        if ( not exists $rails{$crate} ) {
            $rails{$crate} = $rail_num;
            $rail_num++;
        }

        for my $dep ( @{ $event->{deps} || [] } ) {
            $seen_deps{ $dep->{crate} } = 1;

            if ( exists $repressed{ $dep->{crate} } ) {
                unshift @queue, $event;
                unshift @queue, delete $repressed{ $dep->{crate} };
                next event;
            }
        }
        my $crateinfo = $rdb->crate_info($crate);
        print " " x $rails{$crate};
        if ( $event->{action} eq 'create' or $event->{action} eq 'publish' ) {
            my $msrv = $crateinfo->srv_min_version( $event->{version} );
            if ( not defined $msrv ) {
                $msrv = " [UNUSABLE]";
            }
            elsif ( $msrv eq '1.0.0' ) {
                $msrv = "";
            }
            else {
                $msrv = sprintf " (MSRV:%s)", join q[.],
                  splice @{ [ split /[.]/, $msrv ] }, 0, 2;
            }

            print "+ "
              . $crate . ' '
              . $event->{version}
              . $msrv . ' @ '
              . $event->{timestamp};
        }
        if ( $event->{action} eq 'yank' ) {
            print '- '
              . $crate . ' '
              . $event->{version} . ' @ '
              . $event->{timestamp};
        }
        if ( $event->{action} eq 'unyank' ) {
            print '< '
              . $crate . ' '
              . $event->{version} . ' @ '
              . $event->{timestamp};
        }
        print "\n";
    }
}
