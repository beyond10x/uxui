# slot's job pages on the uxui themes

What slot's web view changes so that its job list (`jobs.list`) and job detail (`jobs.detail`) take their
column widths, one-line cells, class hooks, labels beside values and colours per value from `dense.css`
and `slot-jobs.css`. It answers slot's G9 (with F7 and F17), F9, the colour per value of G11 (F12) and the
badge colour of F13, all without an `ess-ui/1` key. Slot's document `spec/ui/slot.ui.yaml` changes in
one place only, step 9: three host slots go, since the theme draws what they drew.
Written against slot's branch `ess-ui-job-views` (commit `a3c91b6`): slot's generator `slot-docs
ui-pages` writes the pages onto the components of `ui/src/components/spec/`.

## Changes

1. Copy `themes/dense.css` and `themes/slot-jobs.css` of uxui to `ui/src/uxui/`, each with a first line
   naming the uxui commit it came from.
2. `ui/src/main.ts`, after `import './style.css';`:

   ```ts
   import './uxui/dense.css';
   import './uxui/slot-jobs.css';
   ```

3. `ui/src/lib/spec-format.ts`, a new function:

   ```ts
   /** A value as the theme's `data-uxui-value`: a scalar as text; nothing for a list, a map or no value. */
   export function themeValue(value: unknown): string | undefined {
     if (value === '' || value === null || value === undefined) return undefined;
     return typeof value === 'string' || typeof value === 'number' || typeof value === 'boolean' ? String(value) : undefined;
   }
   ```

4. `ui/src/components/spec/SpecTable.vue`, template:

   ```vue
   <section class="spec-table uxui-dense-frame" :data-ui-path="path">
   …
   <table class="jobs spec uxui-dense">
   …
   <th v-for="c in columns" :key="c.field + c.label" :class="['uxui-field-' + c.field, 'uxui-as-' + (c.as ?? 'text'), { num: isNumeric(c) }]">{{ c.label }}</th>
   …
   <td v-for="c in columns" :key="c.field + c.label" :class="['uxui-field-' + c.field, 'uxui-as-' + (c.as ?? 'text'), { num: isNumeric(c) }]"
     :data-uxui-value="themeValue(row[c.field])" :title="c.as === 'list' ? formatValue(row[c.field], c) : undefined">
     <SpecHost v-if="c.host" :name="c.host" :row="row" :params="{}" />
     <button v-else-if="c.field === rowKey && openLabel" …>{{ formatValue(row[c.field], c) }}</button>
     <span v-else-if="c.as === 'badge'" class="uxui-badge">{{ formatValue(row[c.field], c) }}</span>
     <template v-else>{{ formatValue(row[c.field], c) }}</template>
   </td>
   …
   <tr v-if="!shown.length" class="empty uxui-empty"><td :colspan="columns.length">{{ emptyText }}</td></tr>
   ```

   The `v-else-if="c.tones"` span goes: the theme colours the value. Import `themeValue` from
   `../../lib/spec-format.ts` (`toneOf` is no longer used here). In `<style scoped>` delete every rule
   from `.spec .f-id` to `.tone-warning` and the `@container (max-width: 1109px)` block; keep `section`
   (its `container-type` may go: `uxui-dense-frame` sets it) and the `.pager` rules.

5. `ui/src/components/spec/SpecRecord.vue`, both field lists:

   ```vue
   <dl class="uxui-dense">
     <template v-for="c in present(fields)" :key="c.field">
       <dt :class="['uxui-field-' + c.field, 'uxui-as-' + (c.as ?? 'text')]">{{ c.label }}</dt>
       <dd :class="['uxui-field-' + c.field, 'uxui-as-' + (c.as ?? 'text')]" :data-uxui-value="themeValue(row[c.field])">
         <SpecHost v-if="c.host" :name="c.host" :row="row" :params="{}" />
         <span v-else-if="c.as === 'badge'" class="uxui-badge">{{ formatValue(row[c.field], c) }}</span>
         <template v-else>{{ formatValue(row[c.field], c) }}</template>
       </dd>
     </template>
   </dl>
   ```

   (`present(g.fields)` in the second list.) Import `themeValue`; `toneOf` is no longer used. In
   `<style scoped>` delete the `dl`, `dt`, `dd`, `dd.as-list, dd.as-map` and `.tone-*` rules; keep
   `section` and `summary`.

6. `ui/src/components/spec/SpecPage.vue`: a prop `badgeField?: string;`, the badge

   ```vue
   <span v-if="badge" class="uxui-badge" :class="badgeField ? 'uxui-field-' + badgeField : undefined" :data-uxui-value="badge">{{ badge }}</span>
   ```

   and in `<style scoped>` the `.state` rule deleted.

7. `crates/slot-docs/src/ui_pages.rs`, where the header badge is written (line 1044):

   ```rust
   let _ = write!(header_attrs, " :badge=\"badge\" badge-field=\"{}\"", attr(field));
   ```

   then `task generate`: `ui/src/generated/pages/JobsDetail.vue` gets `badge-field="state"`.

8. `ui/src/App.vue`, the `watchEffect` that sets `data-theme` sets the uxui mode to the same value:

   ```ts
   watchEffect(() => {
     const mode = view.value === 'cluster' && !specRoute.value ? theme.value : 'light';
     document.documentElement.dataset.theme = mode;
     document.documentElement.dataset.uxuiMode = mode;
   });
   ```

   The job pages stay light: slot's own page styles (`ui/src/style.css`: `--bg`, `--fg`, the header)
   are light only, so `mode` stays `'light'` on them until slot has dark values of its own variables.
   `dense.css` and `slot-jobs.css` carry a dark mode for that day.

9. `spec/ui/slot.ui.yaml`: the three `toolchain` columns of `jobs.list` (sections `running`, `queued`,
   `done`) drop `host`, `stands_in_for` and `reason`, so each reads `{field: toolchain, label: tool}`;
   `RunningTool`, `QueuedTool` and `DoneTool` leave `spec/ui/slot.hosts` and `ui/src/hosts.ts`; then
   `task generate`. Required: the theme draws the colour mark before every toolchain cell with a
   `data-uxui-value`, and the host's `ToolDot` would draw a second one. It is a change to slot's
   document, not to `ess-ui/1`, and it removes three of the eleven host names of F14. The cost: a
   toolchain other than `rust` and `go` shows the grey of `other`, not the hashed colour `tcColor`
   gives it (CSS cannot hash a value).

## Check

`themes/examples/slot-jobs.html` is the markup the steps above produce, with made-up rows.
