# TUI Design

## Visual Direction

WorldHunt is a colored terminal application, not a monochrome ASCII drawing.
The map uses Unicode Braille characters, allowing one terminal cell to
represent a 2 by 4 grid of map samples.

The design prioritizes a recognizable map, readable clues, and restrained
decoration. The game must remain understandable without relying on color alone.

## Layout

Wide terminals show the map and attempt history side by side:

```text
+ WorldHunt ----------------------+ Attempts --------+
|                                 | Canada   4,120 km |
|            WORLD MAP            | Spain    1,230 km |
|                                 | France     Borders |
+---------------------------------+------------------+
| Guess a country: _                    Esc: quit     |
+----------------------------------------------------+
```

Narrow terminals stack a compact header, map, recent attempts, and input. The
map preserves its world aspect ratio and is centered inside the available
space. A terminal below the supported minimum displays a resize message rather
than a broken layout.

The supported minimum is `48x20`. At width `90` and above, map and history are
side by side; below it they stack. Smaller areas show a recovery message without
mutating game or navigation state.

## Controls

| Input | Action |
| --- | --- |
| Printable text | Edit the country guess |
| Backspace | Delete the previous input character |
| Tab | Cycle through country suggestions |
| `+` / `-` | Zoom in or out |
| Mouse wheel | Zoom in or out |
| Arrow keys | Pan the map |
| Enter | Complete the selected suggestion, or submit the current guess |
| `/hint` + Enter | Reveal one more target-name letter without an attempt |
| `/surrender` + Enter | Reveal the target and end the current game |
| Esc | Quit |
| Ctrl+C | Quit |
| N | Start a new game after winning |

Typing remains focused on the country field while a game is active. Letter keys
remain available for country names, including `q`; arrow keys provide panning
and `q` is not a global quit key.

After two normalized characters, the guess field shows up to five matching
country names, separated by `|`. An alias that matches the prefix is shown and
completed in preference to a longer canonical name for the same country. `Tab`
cycles the selection, which is shown with inverse video. `Enter` completes the
selection; a subsequent `Enter` submits the exact current text.
When every suggestion fits the field, their order remains fixed and only the
selected item changes. On narrower terminals, the visible list rotates from
the selected item so it is never hidden.
Recoverable messages take precedence over suggestions.

## Map States

- Water uses a dark, low-contrast background.
- Unguessed playable land uses neutral gray.
- Non-playable territories use a distinct neutral gray.
- Borders use a restrained contrasting shade.
- Guessed countries retain their distance color for the full game.
- The correctly guessed target changes to a distinct winning green.
- A guessed country that disappears during downsampling receives a marker at
  its visual anchor.

## Distance Palette

Distance colors use stable absolute ranges so a color has the same meaning in
every game. The initial bands are subject to visual testing:

| Territorial distance | Visual intensity |
| --- | --- |
| 8,000 km or more | Very muted red |
| 4,000-7,999 km | Dark red |
| 2,000-3,999 km | Medium red |
| 1,000-1,999 km | Vivid red |
| 500-999 km | Intense red |
| 1-499 km | Highest red intensity |
| Shared border | Maximum proximity style |
| Correct target | Winning green |

The final palette must preserve an ordered difference in both truecolor and
ANSI 256-color modes. Distance text in the history makes the clue usable for
players who cannot distinguish all shades.

## Color Capability

The renderer provides a truecolor palette and an ANSI 256-color palette.
`WORLDHUNT_COLOR=truecolor|ansi256|mono` selects an explicit mode; an explicit
valid selection takes precedence over `NO_COLOR`. `NO_COLOR` otherwise selects
monochrome, as do unknown overrides when it is present. Monochrome uses text
and non-color styles: target is bold, a shared border is underlined, and the
nearest distance band is reversed.

Incorrect capability detection must degrade appearance, not break gameplay.

## Map Navigation

The MVP starts centered on Spain. Zoom changes between the embedded zoom-zero
and zoom-one vector tiles; longitude wraps while latitude remains clamped to
the available source coverage. Resize redraws the current viewport without
resetting navigation or game state.

After an accepted guess, the map centers on that country's visual anchor while
preserving the current zoom. Empty, unknown, and repeated inputs do not move the
camera.

## Responsive Rendering

The renderer samples the current Web Mercator viewport into the largest
available Braille grid. Each cell carries eight samples. Resizing recomputes
the viewport without rebuilding world data or resetting navigation or game
state.

The attempt list shows as much history as fits. When it overflows, the most
recent attempts remain visible and the total attempt count remains available.

## Messages and Victory

Unknown and duplicate guesses produce concise, non-fatal messages near the
input. Messages disappear or are replaced by the next relevant action.

Victory clearly names the target and attempt count without obscuring the final
map. `/hint` progressively reveals target-name letters without adding an
attempt. `/surrender` reveals the target without recording a win. Both victory
and surrender offer a new game and quitting.
