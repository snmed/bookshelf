<script lang="ts">
  import { _ } from 'svelte-i18n';
</script>

<svelte:head>
  <title>{$_('labels.settings')}</title>
</svelte:head>

<h1>{$_('labels.settings')}</h1>
<fieldset>
  <legend>{$_('labels.settings')}</legend>
  <hr />

  <input type="radio" id="kraken" name="monster" value="K" />
  <label for="kraken">Kraken</label><br />

  <input type="radio" id="sasquatch" name="monster" value="S" />
  <label for="sasquatch">Sasquatch</label><br />

  <input type="radio" id="mothman" name="monster" value="M" />
  <label for="mothman">Mothman</label>
</fieldset>
<div class="prose max-w-none">
  <h2 class="mb-1">{$_('labels.settings')}</h2>
  <hr />
  <p>
    For years parents have espoused the health benefits of eating garlic bread
    with cheese to their children, with the food earning such an iconic status
    in our culture that kids will often dress up as warm, cheesy loaf for
    Halloween.
  </p>
  <p>
    But a recent study shows that the celebrated appetizer may be linked to a
    series of rabies cases springing up around the country.
  </p>
  <input
    class="input underline"
    id="greet-input"
    placeholder="Enter a name..."
  />
  <input
    type="text"
    placeholder="Type here"
    class="input input-bordered input-sm w-full max-w-xs"
  />
  <!-- ... -->
</div>
