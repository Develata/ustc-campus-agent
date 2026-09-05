import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import vm from 'node:vm';
import test from 'node:test';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const source = readFileSync(path.join(root, 'apps/ustc-agentd/src/web/course-editor.js'), 'utf8');
const fixtureBytes = readFileSync(path.join(root, 'market/fixtures/course-planning/minimal-v0.json'));
const fixture = JSON.parse(fixtureBytes);
const resource = JSON.parse(readFileSync(path.join(root, 'market/packages/ustc.opportunity-graph/components/course-planning-resource-pack.json')));
const context = vm.createContext({});
vm.runInContext(source, context);
const editor = context.UcaCourseEditor;
const plain = (value) => JSON.parse(JSON.stringify(value));
const draft = () => plain(editor.defaultDraft());
const invalid = (value, field) => assert.throws(() => editor.validateDraft(value),
  (error) => error.code === 'invalid_course_draft' && (!field || error.field === field));

test('projection matches the exact package-bound fixture, excluding only lower-authority mirror rows', () => {
  assert.equal(createHash('sha256').update(fixtureBytes).digest('hex'), resource.resourceSha256);
  assert.equal(fixture.source_revision, 'synthetic-course-planning-v0');
  const records = fixture.courses.filter((course) => course.source_id !== 'icourse-mirror-synthetic');
  assert.equal(new Set(records.map((course) => course.code)).size, records.length);
  assert.ok(records.every((course) => course.available));
  assert.deepEqual(plain(editor.catalog), records.map(({ code, title, credits, identity_status }) => ({ code, title, credits, identity_status })));
  const completed = new Set([...fixture.profile.completed_courses,
    ...records.flatMap((course) => course.prerequisites),
    ...records.filter((course) => course.identity_status === 'verified').map((course) => course.code)]);
  assert.deepEqual([...editor.completedCodes].sort(), [...completed].sort());
  assert.equal(editor.courseLabel('MATH2001'), 'MATH2001 · Real Analysis I');
  assert.equal(editor.courseLabel('MATH1001'), 'MATH1001');
});

test('default draft matches fixture, with independent mutable values', () => {
  const expected = { ...fixture.profile, preference_weights: Object.entries(fixture.profile.preference_weights)
    .map(([course_code, weight]) => ({ course_code, weight })) };
  assert.deepEqual(draft(), expected);
  const first = editor.defaultDraft();
  first.completed_courses.pop();
  first.preference_weights[0].weight = -99;
  assert.deepEqual(draft(), expected);
  assert.deepEqual(plain(editor.validateDraft(draft())), expected);
  assert.ok(Object.isFrozen(editor.catalog) && Object.isFrozen(editor.catalog[0]));
});

test('validation accepts changed constraints and integer preference extremes without mutating input', () => {
  const value = { completed_courses: ['CS2001'], min_credits: '0', max_credits: '1',
    preference_weights: [{ course_code: 'CS2002', weight: '-100' }, { course_code: 'MATH2003', weight: '100' }] };
  const original = structuredClone(value);
  assert.deepEqual(plain(editor.validateDraft(value)), { completed_courses: ['CS2001'], min_credits: 0, max_credits: 1,
    preference_weights: [{ course_code: 'CS2002', weight: -100 }, { course_code: 'MATH2003', weight: 100 }] });
  assert.deepEqual(value, original);
  // This draft can be infeasible. The browser must not pretend to plan or reject it as impossible.
  assert.equal(editor.validateDraft({ ...draft(), min_credits: 65535, max_credits: 65535 }).max_credits, 65535);
});

test('reject blank, fractional, nonfinite, exponent and coercible-object credit input', () => {
  for (const value of ['', ' ', '1.5', 1.5, '1e1', '0x10', NaN, Infinity, true, null, {}, [], undefined]) {
    invalid({ ...draft(), min_credits: value }, 'min_credits');
    invalid({ ...draft(), max_credits: value }, 'max_credits');
  }
  invalid({ ...draft(), min_credits: -1 }, 'min_credits');
  invalid({ ...draft(), max_credits: 0 }, 'max_credits');
  invalid({ ...draft(), min_credits: 13 }, 'max_credits');
  invalid({ ...draft(), max_credits: 65536 }, 'max_credits');
});

test('reject unsupported or duplicate completed codes without normalizing identifiers', () => {
  for (const code of ['NOPE9999', 'MATH1001 ', 'math1001', '<script>', 'PE2001', null]) {
    invalid({ ...draft(), completed_courses: [code] }, 'completed_courses');
  }
  invalid({ ...draft(), completed_courses: ['CS1001', 'CS1001'] }, 'completed_courses');
  invalid({ ...draft(), completed_courses: 'CS1001' }, 'completed_courses');
  assert.deepEqual(plain(editor.validateDraft({ ...draft(), completed_courses: [] })).completed_courses, []);
});

test('reject unsupported, unresolved, duplicate and unbounded preferences', () => {
  for (const code of ['CS1001', 'PE2001', 'NOPE9999', 'CS2001 ', null]) {
    invalid({ ...draft(), preference_weights: [{ course_code: code, weight: 1 }] }, 'preference_weights');
  }
  for (const weight of [-101, 101, 1.1, '', '2e1', Infinity, null, {}, true]) {
    invalid({ ...draft(), preference_weights: [{ course_code: 'CS2001', weight }] }, 'weight:CS2001');
  }
  invalid({ ...draft(), preference_weights: [{ course_code: 'CS2001', weight: 1 }, { course_code: 'CS2001', weight: 2 }] }, 'preference_weights');
  for (const entries of [null, {}, [null]]) invalid({ ...draft(), preference_weights: entries }, 'preference_weights');
  assert.deepEqual(plain(editor.validateDraft({ ...draft(), preference_weights: [] })).preference_weights, []);
});

test('closed shape output excludes consent, profile identity and supplied authority fields', () => {
  const value = { ...draft(), consent: true, profile_snapshot_id: 'not-authority', decision: 'planned' };
  assert.deepEqual(Object.keys(editor.validateDraft(value)).sort(), ['completed_courses', 'max_credits', 'min_credits', 'preference_weights']);
  for (const value of [null, undefined, 'draft', []]) invalid(value, 'draft');
});

test('module runs without DOM and fails closed instead of returning a hidden default create draft', () => {
  assert.equal(editor.mount(), false);
  assert.equal(editor.setLocked, editor.setPending);
  assert.throws(() => editor.readDraft(), /尚未挂载/);
  editor.setPending(true, draft());
  assert.throws(() => editor.readDraft(), /尚未挂载/);
  editor.setPending(false);
  assert.doesNotMatch(source, /\bfetch\s*\(|localStorage|sessionStorage|innerHTML/);
});
