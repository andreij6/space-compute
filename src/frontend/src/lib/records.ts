import type { Principal } from '@icp-sdk/core/principal';
import type { Answer, ListRecordsFilter, Protocol, Record_, RecordKind, Result_6 } from '../bindings/aaa';
import type { ActivityItem, ActivityKind, Page_2 } from '../bindings/platform';

export const PROTOCOL_V1 = 1;
export const RECORDS_PAGE_SIZE = 20;

function spaceCamelCase(text: string): string {
  return text.replace(/([a-z])([A-Z])/g, '$1 $2');
}

export function recordKindLabel(kind: RecordKind): string {
  return spaceCamelCase(kind);
}

export function activityKindLabel(kind: ActivityKind): string {
  return spaceCamelCase(kind.__kind__);
}

export function answerLabel(protocol: Protocol | null | undefined, answer: Answer): string {
  const question = protocol?.questions.find((q) => q.id === answer.question_id);
  const option = question?.answers.find((a) => a.id === answer.answer_id);
  return option?.label ?? answer.answer_id;
}

export interface RecordRow {
  id: bigint;
  at: bigint;
  kindLabel: string;
  source: 'aaa' | 'platform';
  record: Record_ | null;
}

export interface RecordsPage {
  items: RecordRow[];
  nextCursor: bigint | null;
  source: 'aaa' | 'platform';
}

export interface RecordsListActor {
  list_records(filter: ListRecordsFilter): Promise<Result_6>;
}
export interface ActivityFallbackActor {
  list_aaa_activity(aaa: Principal, cursor: bigint | null, limit: number): Promise<Page_2>;
}

function fromAaaPage(items: Record_[]): RecordRow[] {
  return items.map((r) => ({ id: r.seq, at: r.at, kindLabel: recordKindLabel(r.kind), source: 'aaa' as const, record: r }));
}

function fromPlatformPage(items: ActivityItem[]): RecordRow[] {
  return items.map((a) => ({ id: a.id, at: a.at, kindLabel: activityKindLabel(a.kind), source: 'platform' as const, record: null }));
}

export async function loadRecordsPage(
  aaa: RecordsListActor,
  platform: ActivityFallbackActor,
  aaaId: Principal,
  cursor: bigint | null,
  limit: number = RECORDS_PAGE_SIZE,
): Promise<RecordsPage> {
  let aaaOkItems: { items: Record_[]; next_cursor?: bigint } | null = null;
  try {
    const result = await aaa.list_records({ cursor: cursor ?? undefined, limit });
    if (result.__kind__ === 'Ok') aaaOkItems = result.Ok;
  } catch {
    aaaOkItems = null;
  }
  if (aaaOkItems) {
    return { items: fromAaaPage(aaaOkItems.items), nextCursor: aaaOkItems.next_cursor ?? null, source: 'aaa' };
  }
  const page = await platform.list_aaa_activity(aaaId, cursor, limit);
  return { items: fromPlatformPage(page.items), nextCursor: page.next_cursor ?? null, source: 'platform' };
}
