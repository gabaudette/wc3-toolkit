Follow the lifetime of the resource your code owns, from allocation to its final use and
release. Jump to [timers](#timers), [groups](#groups) or [locations](#locations).

The diagrams show ownership stages, rather than exact engine scheduling. Each has an
**In words** caption with the same flow. Expand **Code** to see the complete existing recipe
in your selected Lua/JASS language, including its setup and cleanup requirements.

## Read the ownership labels

<span class="lifecycle-key lifecycle-key-owned">Owned handle</span> Your system must arrange
its release when finished. <span class="lifecycle-key lifecycle-key-borrowed">Borrowed reference</span>
Reading a getter did not allocate a new object or transfer cleanup responsibility.

For example, `GetExpiredTimer()` reads the timer associated with the running expiry callback;
it does not create another timer. The callback can release that timer when it owns completion
of the operation. `GetEnumUnit()` reads an existing unit during a group callback; finishing
the callback does not mean the unit should be removed.

See [ownership and cleanup](../glossary/#ownership-and-cleanup) and
[event responses](../glossary/#event-response) for the underlying terms.

**Review status:** These flows follow the project's existing API guidance and paired
recipes. The recipes are syntax-checked; the diagrams do not establish in-game timing,
reentrancy or invalid-handle behavior. Specific engine caveats remain on the linked API pages.

## Timers

### One-shot timer: finish in the callback

<figure class="lifecycle-diagram">
  <ol class="lifecycle-flow" aria-hidden="true">
    <li class="lifecycle-owned"><span class="lifecycle-kind">Owned timer</span><strong><code>CreateTimer</code></strong><span>Allocate one timer.</span></li>
    <li><span class="lifecycle-kind">Start once</span><strong><code>TimerStart</code></strong><span>Use periodic = false.</span></li>
    <li class="lifecycle-borrowed"><span class="lifecycle-kind">Expiry callback</span><strong>Run the callback</strong><span>Use the expired or captured timer reference.</span></li>
    <li class="lifecycle-release"><span class="lifecycle-kind">Release</span><strong><code>DestroyTimer</code></strong><span>Finish using the owned handle.</span></li>
  </ol>
  <figcaption><strong>In words:</strong> Create a timer, start a one-shot countdown, run the expiry callback, then destroy the owned timer from that callback. GetExpiredTimer returns the same timer. A finished countdown is not automatic handle cleanup.</figcaption>
</figure>

The [TimerStart recipe](../api/common/native/timerstart/#example) destroys its timer after
showing a message. Its JASS callback uses `GetExpiredTimer`; its Lua closure retains the
original timer. Both refer to the same owned resource for that operation. Clear finished
stored references as appropriate; clearing the creator's local reference does not stop a timer.

<!-- lifecycle-example: TimerStart -->

### Periodic timer: retain it until the owner stops

<figure class="lifecycle-diagram">
  <ol class="lifecycle-flow" aria-hidden="true">
    <li class="lifecycle-owned"><span class="lifecycle-kind">Owned timer</span><strong><code>CreateTimer</code></strong><span>Keep one reference for the system.</span></li>
    <li><span class="lifecycle-kind">Start repeating</span><strong><code>TimerStart</code></strong><span>Use periodic = true.</span></li>
    <li class="lifecycle-repeat"><span class="lifecycle-kind">While active</span><strong>Expiry callback</strong><span>Do one tick of work.</span><span class="lifecycle-loop">↺ Next expiry runs another tick</span></li>
    <li class="lifecycle-release"><span class="lifecycle-kind">When the owner stops</span><strong><code>PauseTimer</code><code>DestroyTimer</code></strong><span>Clear the retained reference.</span></li>
  </ol>
  <figcaption><strong>In words:</strong> Create and retain one timer, start it as periodic, and let its callback run each tick while the system is active. When the system ends, its separate stop function pauses and destroys the timer and clears the retained reference.</figcaption>
</figure>

The [heartbeat recipe](../api/common/native/createtimer/#example) has a start function and
`ExampleStopHeartbeat()`. Its guard avoids allocating another timer when it is already
running. Each tick keeps the timer alive; the owner calls the stop function when finished.

Pausing alone retains the timer. Destroying it on the first tick would end the repeating
operation. The linked [DestroyTimer caveat](../api/common/native/destroytimer/#unverified-behavior)
does not promise cancellation of callback code that is already running.

<!-- lifecycle-example: CreateTimer -->

## Groups

### Scratch group: release the collection after processing

<figure class="lifecycle-diagram">
  <ol class="lifecycle-flow" aria-hidden="true">
    <li class="lifecycle-owned"><span class="lifecycle-kind">Owned group</span><strong><code>CreateGroup</code></strong><span>Allocate a scratch collection.</span></li>
    <li><span class="lifecycle-kind">Populate</span><strong>Enumerate units</strong><span>Fill the existing group with references.</span></li>
    <li class="lifecycle-borrowed"><span class="lifecycle-kind">Borrowed unit</span><strong><code>ForGroup</code><code>GetEnumUnit</code></strong><span>Process the current member.</span></li>
    <li class="lifecycle-release"><span class="lifecycle-kind">Release group</span><strong><code>DestroyGroup</code></strong><span>The units stay in the map.</span></li>
  </ol>
  <figcaption><strong>In words:</strong> Create a group, populate it with GroupEnumUnitsOfPlayer, process its members through ForGroup and GetEnumUnit, then destroy the group. Your code owns the collection, while the member units are borrowed references.</figcaption>
</figure>

The [ForGroup recipe](../api/common/native/forgroup/#example) issues a stop order to each
member and then releases the collection. Use `GetEnumUnit` inside the callback. There is
no new unit to remove just because enumeration finished.

For a loop that removes processed members, follow the
[FirstOfGroup recipe](../api/common/native/firstofgroup/#example), which calls
`GroupRemoveUnit` before the next iteration. Read its loop and enumeration caveats before
changing group membership during processing.

<!-- lifecycle-example: ForGroup -->

### Reusable group: clear membership, then populate again

<figure class="lifecycle-diagram">
  <ol class="lifecycle-flow" aria-hidden="true">
    <li class="lifecycle-owned"><span class="lifecycle-kind">Owned group</span><strong><code>CreateGroup</code></strong><span>Allocate once for its intended lifetime.</span></li>
    <li class="lifecycle-borrowed"><span class="lifecycle-kind">Use members</span><strong><code>GroupAddUnit</code></strong><span>Add an existing unit.</span></li>
    <li class="lifecycle-repeat"><span class="lifecycle-kind">Keep the collection</span><strong><code>GroupClear</code></strong><span>Discard old membership.</span><span class="lifecycle-loop">↺ Add members again for the next use</span></li>
    <li class="lifecycle-release"><span class="lifecycle-kind">When the owner finishes</span><strong><code>DestroyGroup</code></strong><span>Release even an empty group.</span></li>
  </ol>
  <figcaption><strong>In words:</strong> Create one group, add members, clear it when old membership is no longer needed, and add members again for another use. Destroy the collection at the end of its intended lifetime. Clearing and destroying the group do not remove its units.</figcaption>
</figure>

The [GroupClear recipe](../api/common/native/groupclear/#example) demonstrates reuse within
one function. A retained system can use the same distinction over a longer lifetime, but
must keep an explicit owner and final cleanup path. Do not enumerate into a destroyed group.

<!-- lifecycle-example: GroupClear -->

## Locations

### Temporary location: release the point, keep the unit

<figure class="lifecycle-diagram">
  <ol class="lifecycle-flow" aria-hidden="true">
    <li class="lifecycle-owned"><span class="lifecycle-kind">Owned location</span><strong><code>GetUnitLoc</code></strong><span>Allocate a point from a borrowed unit.</span></li>
    <li><span class="lifecycle-kind">Use the point</span><strong><code>GetLocationX</code><code>GetLocationY</code></strong><span>Read the needed values.</span></li>
    <li class="lifecycle-release"><span class="lifecycle-kind">After the last use</span><strong><code>RemoveLocation</code></strong><span>Release the location.</span></li>
    <li><span class="lifecycle-kind">Finish references</span><strong>Stop using the point</strong><span>Clear stored references if retained.</span></li>
  </ol>
  <figcaption><strong>In words:</strong> GetUnitLoc allocates a location from an existing unit. Read the point's coordinates, remove the location after its last use, and finish using its references. The source unit is borrowed and stays in the map.</figcaption>
</figure>

The [GetUnitLoc recipe](../api/common/native/getunitloc/#example) keeps X/Y as numbers after
releasing the point. Its JASS version clears the local handle; its Lua local reference ends
with the function. If your system stores the point elsewhere, clear those finished references
as well. Assigning `null` or `nil` alone does not call `RemoveLocation`.

[Location](../api/common/native/location/) also allocates an owned point, from known X/Y
numbers, and has the same release requirement. The
[known-point recipe](../api/common/native/removelocation/#example) shows that version.
A point is a snapshot, not a reference that automatically follows a moving unit. Repeated
`GetUnitLoc` calls allocate new locations, including for the same unit.

<!-- lifecycle-example: GetUnitLoc -->

### Numeric coordinates: no location to release

<figure class="lifecycle-diagram">
  <ol class="lifecycle-flow lifecycle-flow-short" aria-hidden="true">
    <li class="lifecycle-borrowed"><span class="lifecycle-kind">Borrowed unit</span><strong>Existing unit</strong><span>Keep its normal owner and lifetime.</span></li>
    <li><span class="lifecycle-kind">Read numbers</span><strong><code>GetUnitX</code><code>GetUnitY</code></strong><span>No location is allocated.</span></li>
    <li><span class="lifecycle-kind">Use X/Y</span><strong>Plain numeric values</strong><span>No RemoveLocation call for these numbers.</span></li>
  </ol>
  <figcaption><strong>In words:</strong> Read an existing unit's X/Y coordinates directly and use the returned numbers. This allocates no location, so there is no point handle to remove. The unit's ownership does not change.</figcaption>
</figure>

Use the [numeric-coordinate recipe](../api/common/native/getunitx/#example) when a calculation
or receiving API only needs numbers. Both stored numbers and allocated locations describe
the position when read; read again when an updated position is needed. The
[coordinates/location comparison](../common-confusions/#coordinates-versus-location-handles)
explains choosing the right form.

## Before adding cleanup to a callback

Identify the resource you own and which call allocated it. Reading a unit through
[GetTriggerUnit](../api/common/native/gettriggerunit/) or
[GetEnumUnit](../api/common/native/getenumunit/) does not create a new unit for your callback
to clean up. Likewise, `GetExpiredTimer` reads a timer that already exists; release it only
when your operation owns completion of that timer.

Do not destroy resources borrowed from another system merely because your current callback
has finished. Follow the [practical API ownership notes](../api/practical/), and capture
needed event values before deferring work to another context. Engine scheduling precision,
reentrant teardown and invalid-handle behavior are not established by these diagrams.
