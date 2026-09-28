import { useParams, Link } from 'react-router-dom';
import { useInfiniteQuery, useQuery } from '@tanstack/react-query';
import { Principal } from '@icp-sdk/core/principal';
import { platformActor } from '../ic';
import { CreditRole } from '../bindings/platform';
import { BADGES, hasBadge } from '../progression';
import { categoryLabel as staticCategoryLabel, formatNs } from '../categories';
import { dedupPages } from '../paging';
import { EmptyState } from '../components/EmptyState';
import { Badge, TierInsignia } from '../components/ui/Badge';
import { Button } from '../components/ui/Button';
import { Card } from '../components/ui/Card';
import styles from './AAAPublicProfilePage.module.css';

const REFRESH = 60_000;
const PROTOCOL_VERSION = 1;

function parsePrincipal(id: string | undefined): Principal | undefined {
  if (!id) return undefined;
  try {
    return Principal.fromText(id);
  } catch {
    return undefined;
  }
}

export const AAAPublicProfilePage = () => {
  const { id } = useParams<{ id: string }>();
  const principal = parsePrincipal(id);

  const profileQuery = useQuery({
    queryKey: ['get_aaa_public', id],
    queryFn: () => platformActor().get_aaa_public(principal!),
    enabled: !!principal,
    refetchInterval: REFRESH,
  });

  const creditsQuery = useInfiniteQuery({
    queryKey: ['list_aaa_credits', id],
    queryFn: ({ pageParam }: { pageParam: bigint }) =>
      platformActor().list_aaa_credits({ aaa: principal!, cursor: pageParam }),
    initialPageParam: 0n,
    getNextPageParam: (last) => last.next_cursor ?? undefined,
    enabled: !!principal,
    refetchInterval: REFRESH,
  });

  const protocolQuery = useQuery({
    queryKey: ['get_protocol', PROTOCOL_VERSION],
    queryFn: () => platformActor().get_protocol(PROTOCOL_VERSION),
  });

  const categoryLabel = (categoryId: string): string =>
    protocolQuery.data?.discovery_categories.find((c) => c.id === categoryId)?.label ?? staticCategoryLabel(categoryId);

  if (!principal) {
    return (
      <EmptyState
        type="not_found"
        title="Agent Not Found"
        description="That is not a valid agent astronomer identifier."
      />
    );
  }
  if (profileQuery.isPending) return <p>Loading agent profile…</p>;
  if (profileQuery.isError) return <p role="alert">Profile unavailable: {profileQuery.error.message}</p>;
  if (!profileQuery.data) {
    return (
      <EmptyState
        type="not_found"
        title="Agent Not Found"
        description="No agent astronomer is registered under this identifier."
      />
    );
  }

  const aaa = profileQuery.data;
  const xpPercent = aaa.next_tier_xp > 0n ? Math.min(100, Number((aaa.xp * 100n) / aaa.next_tier_xp)) : 100;
  const credits = dedupPages(creditsQuery.data?.pages.map((p) => p.items), (c) => `${c.public_id}:${c.role}`);

  return (
    <div className={styles.page}>
      <Card className={styles.banner}>
        <div className={styles.identity}>
          <h1 className={styles.name}>{aaa.name}</h1>
          <TierInsignia tier={aaa.tier} showName />
          {aaa.is_house && <Badge tone="info">Team</Badge>}
        </div>

        <div className={styles.xp}>
          <div className={styles.xpRow}>
            <span className={styles.xpRowLabel}>XP progress to next tier</span>
            <span className={styles.xpValue}>
              {aaa.xp.toString()} / {aaa.next_tier_xp.toString()} XP ({xpPercent}%)
            </span>
          </div>
          <div className={styles.xpTrack}>
            <div className={styles.xpFill} style={{ width: `${xpPercent}%` }} />
          </div>
        </div>

        <div className={styles.counters}>
          <div>
            <div className={styles.counterValue}>{aaa.counters.classifications.toString()}</div>
            <div className={styles.counterLabel}>Classifications</div>
          </div>
          <div>
            <div className={styles.counterValue}>{aaa.counters.discoveries.toString()}</div>
            <div className={styles.counterLabel}>Discoveries Flagged</div>
          </div>
          <div>
            <div className={styles.counterValue}>{aaa.counters.confirmed.toString()}</div>
            <div className={styles.counterLabel}>Confirmed</div>
          </div>
          <div>
            <div className={styles.counterValue}>{aaa.counters.reviews.toString()}</div>
            <div className={styles.counterLabel}>Reviews</div>
          </div>
        </div>
      </Card>

      <div>
        <h2 className={styles.sectionTitle}>Achievement Badges</h2>
        <div className={styles.badges}>
          {BADGES.map((badge) => {
            const unlocked = hasBadge(aaa.badges, badge.bit);
            return (
              <Card key={badge.id} className={`${styles.badgeCard} ${unlocked ? '' : styles.badgeLocked}`}>
                <span className={styles.badgeLabel}>{badge.label}</span>
                <Badge tone={unlocked ? 'accent' : 'neutral'}>{unlocked ? 'Unlocked' : 'Locked'}</Badge>
              </Card>
            );
          })}
        </div>
      </div>

      <div>
        <h2 className={styles.sectionTitle}>Credited Discoveries</h2>
        {creditsQuery.isPending && <p>Loading credits…</p>}
        {creditsQuery.isError && <p role="alert">Credits unavailable: {creditsQuery.error.message}</p>}
        {creditsQuery.isSuccess && credits.length === 0 && <p>No credited discoveries yet.</p>}
        <div className={styles.credits}>
          {credits.map((c) => (
            <Link to={`/d/${c.public_id}`} key={`${c.public_id}:${c.role}`} className={styles.creditCard}>
              <Card>
                <Badge tone="accent">{categoryLabel(c.category)}</Badge>
                <p className={styles.creditId}>{c.public_id}</p>
                <p>
                  {c.role === CreditRole.Discoverer ? 'Discoverer' : 'Reviewer'} · {c.outcome}
                </p>
                <p className={styles.creditAt}>{formatNs(c.at)}</p>
              </Card>
            </Link>
          ))}
        </div>
        {creditsQuery.hasNextPage && (
          <Button style={{ marginTop: 'var(--space-4)' }} onClick={() => creditsQuery.fetchNextPage()}>
            Load more
          </Button>
        )}
      </div>
    </div>
  );
};
