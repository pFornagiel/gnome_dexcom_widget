import GObject from 'gi://GObject';
import St from 'gi://St';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Clutter from 'gi://Clutter';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';

const ARROW_ICON_MAP = {
    '↑↑': 'chevrons-up',
    '↑':  'move-up',
    '↗':  'move-up-right',
    '→':  'move-right',
    '↘':  'move-down-right',
    '↓':  'move-down',
    '↓↓': 'chevrons-down',
};

const GlucoseIndicator = GObject.registerClass(
class GlucoseIndicator extends PanelMenu.Button {
    _init(extensionPath) {
        super._init(0.0, 'Glucose Indicator', false);

        this._extensionPath = extensionPath;

        // Horizontal box: label + icon + diff label
        this._box = new St.BoxLayout({
            style_class: 'panel-status-indicators-box',
        });

        this._valueLabel = new St.Label({
            text: 'Loading...',
            y_align: Clutter.ActorAlign.CENTER,
        });

        this._icon = new St.Icon({
            style_class: 'system-status-icon',
            icon_size: 16,
            y_align: Clutter.ActorAlign.CENTER,
        });
        this._icon.visible = false;

        this._diffLabel = new St.Label({
            text: '',
            y_align: Clutter.ActorAlign.CENTER,
        });

        this._box.add_child(this._valueLabel);
        this._box.add_child(this._icon);
        this._box.add_child(this._diffLabel);
        this.add_child(this._box);

        this.timeoutId = null;
        this.statusFile = this._findStatusFile();
    }

    _findStatusFile() {
        const runtimeDir = GLib.getenv('XDG_RUNTIME_DIR');
        if (runtimeDir) {
             const path = `${runtimeDir}/glucose-monitor/status.json`;
             const file = Gio.File.new_for_path(path);
             if (file.query_exists(null)) return path;
        }

        const cacheDir = GLib.getenv('XDG_CACHE_HOME') || `${GLib.get_home_dir()}/.cache`;
        return `${cacheDir}/glucose-monitor/status.json`;
    }

    /**
     * Set the St.Icon gicon from a local SVG file matching the trend arrow.
     * Falls back to hiding the icon if the arrow is unknown.
     */
    _setTrendIcon(arrow) {
        const baseName = ARROW_ICON_MAP[arrow];
        if (!baseName) {
            this._icon.visible = false;
            return;
        }

        const svgPath = GLib.build_filenamev([this._extensionPath, 'svg', `${baseName}.svg`]);
        const file = Gio.File.new_for_path(svgPath);

        if (file.query_exists(null)) {
            this._icon.gicon = new Gio.FileIcon({ file });
            this._icon.visible = true;
        } else {
            log(`glucose-bar: SVG not found: ${svgPath}`);
            this._icon.visible = false;
        }
    }

    enable() {
        this.timeoutId = GLib.timeout_add_seconds(GLib.PRIORITY_DEFAULT, 30, () => {
             this._update();
             return GLib.SOURCE_CONTINUE;
        });
        this._update(); // Initial update
    }

    disable() {
        if (this.timeoutId) {
            GLib.source_remove(this.timeoutId);
            this.timeoutId = null;
        }
        this._valueLabel.text = 'Disabled';
        this._icon.visible = false;
        this._diffLabel.text = '';
    }

    _update() {
        this.statusFile = this._findStatusFile();
        const file = Gio.File.new_for_path(this.statusFile);

        file.load_contents_async(null, (file, res) => {
            try {
                const [success, contents, etag] = file.load_contents_finish(res);
                if (success) {
                    const decoder = new TextDecoder('utf-8');
                    const jsonString = decoder.decode(contents);
                    try {
                        const data = JSON.parse(jsonString);
                        this._updateLabel(data);
                    } catch (e) {
                        log(`Error parsing JSON: ${e}`);
                        this._valueLabel.text = 'JSON Error';
                        this._icon.visible = false;
                        this._diffLabel.text = '';
                    }
                }
            } catch (e) {
                this._valueLabel.text = 'Waiting...';
                this._icon.visible = false;
                this._diffLabel.text = '';
            }
        });
    }

    _updateLabel(data) {
        // data: { mg_dl, trend_arrow, diff, timestamp }
        this._valueLabel.text = `${data.mg_dl}`;

        // Set the SVG trend icon
        this._setTrendIcon(data.trend_arrow);

        // Diff text
        if (data.diff !== null && data.diff !== undefined) {
            const sign = data.diff >= 0 ? '+' : '';
            this._diffLabel.text = ` (${sign}${data.diff})`;
        } else {
            this._diffLabel.text = '';
        }

        // Check for staleness (> 10 minutes old)
        const now = GLib.DateTime.new_now_utc().to_unix();
        const diffSecs = now - data.timestamp;

        if (diffSecs > 600) {
            this._valueLabel.text += ' (Stale)';
            this._valueLabel.style_class_name = 'glucose-label-stale';
        } else {
            this._valueLabel.style_class_name = '';
        }
    }
});

let indicator;

export default class GlucoseExtension {
    constructor(meta) {
        this._meta = meta;
    }

    enable() {
        // meta.path gives the absolute path to the extension directory
        indicator = new GlucoseIndicator(this._meta.path);
        indicator.enable();
        Main.panel.addToStatusArea('glucose-indicator', indicator);
    }

    disable() {
        if (indicator) {
            indicator.disable();
            indicator.destroy();
            indicator = null;
        }
    }
}
