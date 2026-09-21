    /// Sub-round 2 (2026-09-15): the vanilla 1.16.5 tabbed creative
    /// inventory. clean-room: shape language = the classic (pre-1.19.3)
    /// creative screen — two tab rows on top (6 + 5 tabs, icon-only
    /// folder tabs), a 9x5 slot grid with a right scrollbar, the tab
    /// title above the grid (the Search tab replaces it with a search
    /// field), and the hotbar + destroy slot at the bottom. Reference
    /// facts: the reference wiki's creative-inventory page (live 2026-09-15):
    /// the nine content tabs + Search Items + Survival Inventory; the
    /// 9x5/45-per-page grid with a scrollbar; "A single item can be
    /// grabbed using left-click ... Right-clicking an item also picks
    /// up one item ... Shift-clicking an item grabs a full stack";
    /// "Pressing a number key while hovering over an item instantly
    /// places one full stack of that item into the hotbar slot"; the
    /// destroy slot ("get rid of the held item" by clicking outside or
    /// over another item). Proportions are the engine's established 2x
    /// container geometry (40px slots on the 960x540 canvas). No third-party
    /// asset was read, copied, or traced.
    #[allow(clippy::too_many_arguments)]
    pub fn creative_screen(
        &mut self,
        cursor: (f32, f32),
        atlas: &[u8],
        tab: u8,
        scroll: usize,
        search: &str,
        search_focused: bool,
        items: &[u16],
        hotbar: &[vc_inventory::inventory::ItemStack],
        selected: usize,
        cursor_stack: &vc_inventory::inventory::ItemStack,
        advanced_tooltips: bool,
    ) -> CreativeGeom {
        use vc_blocks::blocks as blk;

        // ---- layout constants (engine 2x container geometry) ----
        let cols = 9usize;
        let vis_rows = 5usize;
        let cell = 40i32;
        let grid_w = cols as i32 * cell + 4;
        let x0 = (UI_W as i32 - grid_w) / 2;
        // panel: title strip 26px; grid 5x40; hotbar row below
        let y0 = 150i32; // grid top
        let title_y = y0 - 26;
        let hot_y = y0 + vis_rows as i32 * cell + 14;
        let px0 = x0 - 14;
        let pw = grid_w + 28;
        let py0 = title_y - 10;
        let ph = (hot_y + 46) - py0;
        // tab strip: two rows (6 + 5), folder tabs attached to the panel
        let tab_w = 76i32;
        let tab_h = 44i32;
        let row1_n = 6i32;
        let row1_x = (UI_W as i32 - row1_n * tab_w) / 2;
        let row2_n = 5i32;
        let row2_x = (UI_W as i32 - row2_n * tab_w) / 2;
        let row2_y = py0 - tab_h + 2;
        let row1_y = row2_y - tab_h + 2;

        let mut geom = CreativeGeom {
            x0,
            y0,
            cell,
            cols,
            vis_rows,
            scroll,
            tabs: [None; 11],
            hotbar: [(0, 0, 0, 0); 9],
            trash: (0, 0, 0, 0),
            search: (0, 0, 0, 0),
            scrollbar: None,
        };

        // ---- tab strip (11 tabs: 9 content + Search + Inventory) ----
        // vanilla tab order: Building, Decoration, Redstone, Transport,
        // Misc, Food, Tools, Combat, Brewing, Search, Inventory
        let tab_labels: [&str; 11] = [
            "BUILDING BLOCKS",
            "DECORATION BLOCKS",
            "REDSTONE",
            "TRANSPORTATION",
            "MISCELLANEOUS",
            "FOODSTUFFS",
            "TOOLS",
            "COMBAT",
            "BREWING",
            "SEARCH ITEMS",
            "INVENTORY",
        ];
        let cx = cursor.0 as i32;
        let cy = cursor.1 as i32;
        let mut hovered_tab: Option<u8> = None;
        for t in 0..11u8 {
            let (tx, ty) = if t < 6 {
                (row1_x + t as i32 * tab_w, row1_y)
            } else {
                (row2_x + (t as i32 - 6) * tab_w, row2_y)
            };
            let selected_tab = t == tab;
            // folder-tab chrome: attached to the panel on selection
            let bg: Color = if selected_tab {
                [58, 58, 62, 245]
            } else {
                [34, 34, 38, 225]
            };
            self.rect(tx, ty, tab_w, tab_h, bg);
            self.frame(tx, ty, tab_w, tab_h, [78, 78, 84, 255]);
            if selected_tab {
                self.frame(tx, ty + 1, tab_w, tab_h - 2, [140, 140, 148, 255]);
                // connect to the panel: erase the bottom edge line
                self.rect(tx + 1, ty + tab_h - 2, tab_w - 2, 2, [58, 58, 62, 245]);
            }
            // the tab icon: the canonical item's tile, 2x blit
            let icon = if t < 9 {
                blk::CREATIVE_TABS[t as usize].icon_block()
            } else if t == 9 {
                // Search tab icon: the compass — engine substitute: the
                // eye of ender (the registry's search-est item; no compass)
                blk::EYE_OF_ENDER
            } else {
                // Inventory tab icon: the player head — engine
                // substitute: the zombie spawn egg (a face-like egg)
                blk::SPAWN_EGG_ZOMBIE
            };
            let tile = blk::def(icon).tiles[0];
            blit_tile(atlas, tile, 2, (tx + (tab_w - 32) / 2) as usize, (ty + 6) as usize, &mut self.px, UI_W);
            // hover highlight + tooltip capture
            if cx >= tx && cx < tx + tab_w && cy >= ty && cy < ty + tab_h {
                self.frame(tx - 1, ty - 1, tab_w + 2, tab_h + 2, [255, 255, 255, 200]);
                hovered_tab = Some(t);
            }
            geom.tabs[t as usize] = Some((tx, ty, tab_w, tab_h));
        }

        // ---- panel ----
        self.rect(px0, py0, pw, ph, [26, 26, 30, 235]);
        self.frame(px0, py0, pw, ph, [60, 60, 66, 255]);
        self.frame(px0 + 1, py0 + 1, pw - 2, ph - 2, [12, 12, 14, 255]);

        // ---- title / search field ----
        if tab == 9 {
            // the Search tab: a text field in the title strip
            let fx = x0;
            let fy = title_y;
            let fw = grid_w;
            let fh = 22;
            let w = Widget {
                id: 0,
                x: fx,
                y: fy,
                w: fw,
                h: fh,
                kind: WidgetKind::TextField {
                    label: String::new(),
                    text: search.to_string(),
                    placeholder: "SEARCH".to_string(),
                    focused: search_focused,
                },
            };
            self.draw_text_field(&w, false);
            geom.search = (fx, fy, fw, fh);
        } else {
            let label = if tab < 9 {
                blk::CREATIVE_TABS[tab as usize].label().to_uppercase()
            } else {
                tab_labels[tab as usize].to_string()
            };
            self.text(x0, title_y + 4, &label, [255, 220, 120, 255], 1);
        }

        // ---- the 9x5 grid ----
        let total_rows = items.len().div_ceil(cols);
        let scroll = scroll.min(total_rows.saturating_sub(vis_rows));
        let first = scroll * cols;
        let last = (first + vis_rows * cols).min(items.len());
        let mut hovered: Option<u16> = None;
        for (i, &b) in items[first..last].iter().enumerate() {
            let col = (i % cols) as i32;
            let row = (i / cols) as i32;
            let sx = x0 + 4 + col * cell;
            let sy = y0 + 4 + row * cell;
            self.rect(sx, sy, 36, 36, [52, 52, 52, 200]);
            self.frame(sx, sy, 36, 36, [24, 24, 24, 255]);
            self.frame(sx + 1, sy + 1, 34, 34, [110, 110, 110, 255]);
            let tile = blk::def(b).tiles[0];
            blit_tile(atlas, tile, 2, (sx + 2) as usize, (sy + 2) as usize, &mut self.px, UI_W);
            if cx >= sx && cx < sx + 36 && cy >= sy && cy < sy + 36 {
                self.frame(sx - 1, sy - 1, 38, 38, [255, 255, 255, 255]);
                hovered = Some(b);
            }
        }
        // empty slots for the unfilled tail of the last page (vanilla
        // shows empty slots, not blank space)
        let tail_start = items.len().saturating_sub(first);
        if tail_start < vis_rows * cols {
            for i in tail_start..(vis_rows * cols) {
                let col = (i % cols) as i32;
                let row = (i / cols) as i32;
                let sx = x0 + 4 + col * cell;
                let sy = y0 + 4 + row * cell;
                self.rect(sx, sy, 36, 36, [40, 40, 44, 160]);
                self.frame(sx, sy, 36, 36, [20, 20, 20, 200]);
            }
        }

        // ---- scrollbar (when the tab overflows one page) ----
        if total_rows > vis_rows {
            let sb_x = px0 + pw - 16;
            let sb_y = y0 + 2;
            let sb_h = vis_rows as i32 * cell;
            self.rect(sb_x, sb_y, 10, sb_h, [16, 16, 18, 220]);
            self.frame(sb_x, sb_y, 10, sb_h, [10, 10, 10, 255]);
            let track = (total_rows - vis_rows).max(1);
            let thumb_h = (((sb_h as f32) * (vis_rows as f32 / total_rows as f32)) as i32).max(24);
            let avail = sb_h - thumb_h;
            let thumb_y = sb_y + ((scroll as f32 / track as f32) * avail as f32) as i32;
            self.rect(sb_x + 1, thumb_y + 1, 8, thumb_h - 2, [150, 150, 155, 230]);
            geom.scrollbar = Some((sb_x, sb_y, 10, sb_h));
        }

        // ---- bottom row: hotbar + destroy slot ----
        for (i, s) in hotbar.iter().take(9).enumerate() {
            let sx = x0 + 4 + i as i32 * cell;
            let sy = hot_y;
            self.rect(sx, sy, 36, 36, [52, 52, 52, 200]);
            self.frame(sx, sy, 36, 36, [24, 24, 24, 255]);
            self.frame(sx + 1, sy + 1, 34, 34, [110, 110, 110, 255]);
            self.draw_stack(s, sx, sy, atlas);
            if i == selected {
                self.frame(sx - 2, sy - 2, 40, 40, [255, 255, 255, 230]);
            }
            geom.hotbar[i] = (sx, sy, 36, 36);
        }
        // destroy slot (trash): bottom-right, X-marked
        {
            let sx = px0 + pw - 46;
            let sy = hot_y;
            self.rect(sx, sy, 36, 36, [70, 40, 40, 210]);
            self.frame(sx, sy, 36, 36, [24, 24, 24, 255]);
            self.frame(sx + 1, sy + 1, 34, 34, [110, 70, 70, 255]);
            // X mark
            for k in 0..12i32 {
                self.set(sx + 12 + k, sy + 12 + k, [200, 90, 90, 255]);
                self.set(sx + 23 - k, sy + 12 + k, [200, 90, 90, 255]);
            }
            geom.trash = (sx, sy, 36, 36);
        }

        // ---- hovered item name (above the hotbar, centered) ----
        let label = hovered
            .map(blk::name)
            .map(|n| {
                if advanced_tooltips {
                    let id: String = n.to_lowercase().replace(' ', "_");
                    format!("{n} (voxelcraft:{id})")
                } else {
                    n.to_string()
                }
            })
            .or_else(|| {
                hovered_tab.map(|t| {
                    // tab tooltip = the vanilla tab label
                    if t < 9 {
                        blk::CREATIVE_TABS[t as usize].label().to_string()
                    } else {
                        tab_labels[t as usize].to_string()
                    }
                })
            })
            .unwrap_or_default();
        if !label.is_empty() {
            let lw = Self::text_width(&label, 1);
            self.text((UI_W as i32 - lw) / 2, hot_y - 16, &label, [255, 255, 255, 255], 1);
        }

        // ---- the held (cursor) stack follows the mouse ----
        if !cursor_stack.is_empty() {
            let hx = cx - 18;
            let hy = cy - 18;
            self.draw_stack(cursor_stack, hx, hy, atlas);
        }

        geom.scroll = scroll;
        geom
    }

