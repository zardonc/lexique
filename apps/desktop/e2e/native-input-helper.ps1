param(
    [ValidateSet('editor', 'focus', 'shortcut', 'position', 'bounds', 'drag', 'monitors', 'close', 'picker-windows', 'picker-controls', 'picker-select', 'picker-cancel')] [string] $Action,
    [string] $Text = 'école',
    [string] $Title = 'Lexique selection editor',
    [int] $SourcePid = 0,
    [byte] $Key = 0x4B,
    [switch] $WithShift,
    [switch] $OcrSample,
    [int] $X = 30,
    [int] $Y = 30,
    [int] $EndX = 0,
    [int] $EndY = 0,
    [string] $SourceExecutable = '',
    [ValidateRange(1, 6000)] [int] $WaitMs = 4000,
    [int] $FilenameControlId = 0
)
$ErrorActionPreference = 'Stop'
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
public static class NativeInputFixture {
    [StructLayout(LayoutKind.Sequential)] public struct Point { public int X, Y; }
    [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
    [StructLayout(LayoutKind.Sequential)] public struct MonitorInfo { public uint Size; public Rect Monitor, Work; public uint Flags; }
    [StructLayout(LayoutKind.Sequential)] public struct Message {
        public IntPtr Window; public uint Id; public UIntPtr WParam; public IntPtr LParam;
        public uint Time; public Point Position; public uint Private;
    }
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr CreateWindowEx(uint ex, string cls, string title, uint style,
        int x, int y, int width, int height, IntPtr parent, IntPtr menu, IntPtr instance, IntPtr data);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr FindWindow(string cls, string title);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr FindWindowEx(IntPtr parent, IntPtr after, string cls, string title);
    [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr window);
    [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] static extern IntPtr SetActiveWindow(IntPtr window);
    [DllImport("user32.dll")] static extern IntPtr GetActiveWindow();
    [DllImport("user32.dll")] static extern IntPtr SetFocus(IntPtr window);
    [DllImport("user32.dll")] static extern bool AttachThreadInput(uint source, uint target, bool attach);
    [DllImport("user32.dll")] static extern bool PeekMessage(out Message message, IntPtr window, uint first, uint last, uint remove);
    [DllImport("kernel32.dll")] static extern uint GetCurrentThreadId();
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode)] static extern IntPtr LoadLibrary(string name);
    [DllImport("user32.dll")] static extern IntPtr SendMessage(IntPtr window, uint message, IntPtr w, IntPtr l);
    [DllImport("user32.dll")] static extern bool IsWindow(IntPtr window);
    [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr window);
    [DllImport("user32.dll")] static extern IntPtr GetWindow(IntPtr window, uint command);
    delegate bool WindowCallback(IntPtr window, IntPtr parameter);
    [DllImport("user32.dll")] static extern bool EnumWindows(WindowCallback callback, IntPtr parameter);
    [DllImport("user32.dll")] static extern bool EnumChildWindows(IntPtr parent, WindowCallback callback, IntPtr parameter);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetClassName(IntPtr window, StringBuilder text, int maximum);
    [DllImport("user32.dll")] static extern int GetDlgCtrlID(IntPtr window);
    [DllImport("user32.dll")] static extern IntPtr GetAncestor(IntPtr window, uint flags);
    [DllImport("user32.dll")] static extern bool IsWindowEnabled(IntPtr window);
    [DllImport("user32.dll", CharSet=CharSet.Unicode, EntryPoint="SendMessageTimeoutW")] static extern IntPtr TimedText(IntPtr window, uint message, UIntPtr w, string text, uint flags, uint timeout, out UIntPtr result);
    [DllImport("user32.dll", CharSet=CharSet.Unicode, EntryPoint="SendMessageTimeoutW")] static extern IntPtr TimedRead(IntPtr window, uint message, UIntPtr w, StringBuilder text, uint flags, uint timeout, out UIntPtr result);
    [DllImport("user32.dll", EntryPoint="SendMessageTimeoutW")] static extern IntPtr TimedMessage(IntPtr window, uint message, UIntPtr w, IntPtr l, uint flags, uint timeout, out UIntPtr result);
    [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr window, out uint process);
    [DllImport("user32.dll")] static extern int GetMessage(out Message message, IntPtr window, uint min, uint max);
    [DllImport("user32.dll")] static extern bool TranslateMessage(ref Message message);
    [DllImport("user32.dll")] static extern IntPtr DispatchMessage(ref Message message);
    [DllImport("user32.dll")] static extern void keybd_event(byte key, byte scan, uint flags, UIntPtr extra);
    [DllImport("user32.dll")] static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
    [DllImport("user32.dll")] static extern bool SetWindowPos(IntPtr window, IntPtr after, int x, int y, int width, int height, uint flags);
    [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr window, out Rect rect);
    [DllImport("user32.dll")] static extern bool GetClientRect(IntPtr window, out Rect rect);
    [DllImport("user32.dll")] static extern bool ClientToScreen(IntPtr window, ref Point point);
    [DllImport("user32.dll")] static extern uint GetDpiForWindow(IntPtr window);
    delegate bool MonitorCallback(IntPtr monitor, IntPtr dc, ref Rect bounds, IntPtr parameter);
    [DllImport("user32.dll")] static extern bool EnumDisplayMonitors(IntPtr dc, IntPtr clip, MonitorCallback callback, IntPtr parameter);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern bool GetMonitorInfo(IntPtr monitor, ref MonitorInfo info);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr window, StringBuilder text, int maximum);
    [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] static extern bool PostMessage(IntPtr window, uint message, IntPtr wParam, IntPtr lParam);
    [DllImport("user32.dll")] static extern void mouse_event(uint flags, uint x, uint y, uint data, UIntPtr extra);
    [DllImport("gdi32.dll", CharSet=CharSet.Unicode)] static extern IntPtr CreateFont(int height, int width, int angle, int orientation,
        int weight, uint italic, uint underline, uint strikeout, uint charset, uint output, uint clip, uint quality, uint pitch, string face);
    public static void Editor(string title, string text, bool ocrSample) {
        SetThreadDpiAwarenessContext(new IntPtr(-4));
        var window = CreateWindowEx(0, "STATIC", title, 0x10CF0000, 30, 30, 900, 430, IntPtr.Zero, IntPtr.Zero, IntPtr.Zero, IntPtr.Zero);
        if (window == IntPtr.Zero) throw new Exception("Could not create owned editor");
        if(LoadLibrary("Msftedit.dll") == IntPtr.Zero) throw new Exception("Windows RichEdit is unavailable");
        var edit = CreateWindowEx(0x200, "RICHEDIT50W", text, 0x50301004, 10, 10, 850, 350, window, IntPtr.Zero, IntPtr.Zero, IntPtr.Zero);
        if(edit == IntPtr.Zero) throw new Exception("Could not create owned RichEdit control");
        if(ocrSample) SendMessage(edit, 0x30, CreateFont(-34, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0, "Segoe UI"), new IntPtr(1));
        if(!SetWindowPos(window, IntPtr.Zero, 0, 0, 0, 0, 0x47)) throw new Exception("Could not show owned editor");
        SetForegroundWindow(window); SetFocus(edit); SendMessage(edit, 0xB1, IntPtr.Zero, new IntPtr(ocrSample ? 0 : text.Length));
        while (IsWindow(window) && GetMessage(out var message, IntPtr.Zero, 0, 0) > 0) {
            TranslateMessage(ref message); DispatchMessage(ref message);
        }
    }
    public static void FocusSource(string title, int process, bool ocrSample) {
        var window = FindWindow(null, title);
        if(process != 0) {
            window = IntPtr.Zero;
            EnumWindows((candidate, _) => {
                GetWindowThreadProcessId(candidate, out var owner);
                if(owner != process || !IsWindowVisible(candidate) || GetWindow(candidate, 4) != IntPtr.Zero) return true;
                window = candidate; return false;
            }, IntPtr.Zero);
        }
        if (window == IntPtr.Zero) throw new Exception("Owned source window not found: " + title);
        PeekMessage(out _, IntPtr.Zero, 0, 0, 0);
        var ownThread = GetCurrentThreadId();
        var sourceThread = GetWindowThreadProcessId(window, out var sourceProcess);
        var foregroundThread = GetWindowThreadProcessId(GetForegroundWindow(), out _);
        var attachForeground = ownThread != foregroundThread && foregroundThread != sourceThread;
        var attachSource = ownThread != sourceThread;
        if(attachForeground && !AttachThreadInput(ownThread, foregroundThread, true)) throw new Exception("Could not attach foreground input");
        if(attachSource && !AttachThreadInput(ownThread, sourceThread, true)) throw new Exception("Could not attach owned source input");
        var edit = FindWindowEx(window, IntPtr.Zero, "RICHEDIT50W", null);
        try {
            if (!SetForegroundWindow(window)) throw new Exception("Could not activate owned source");
            if(edit != IntPtr.Zero) SetFocus(edit);
        } finally {
            if(attachSource) AttachThreadInput(ownThread, sourceThread, false);
            if(attachForeground) AttachThreadInput(ownThread, foregroundThread, false);
        }
        SendMessage(window, 0, IntPtr.Zero, IntPtr.Zero);
        if(edit != IntPtr.Zero) SendMessage(edit, 0xB1, IntPtr.Zero, new IntPtr(ocrSample ? 0 : -1));
        System.Threading.Thread.Sleep(120);
        if (GetForegroundWindow() != window) {
            GetWindowThreadProcessId(GetForegroundWindow(), out var foregroundProcess);
            throw new Exception("Owned source " + sourceProcess + " lost foreground to " + foregroundProcess);
        }
    }
    public static void Shortcut(string title, byte key, int process, bool withShift, bool ocrSample) {
        FocusSource(title, process, ocrSample);
        keybd_event(0x11, 0, 0, UIntPtr.Zero);
        if(withShift) keybd_event(0x10, 0, 0, UIntPtr.Zero);
        keybd_event(0x12, 0, 0, UIntPtr.Zero); keybd_event(key, 0, 0, UIntPtr.Zero);
        keybd_event(key, 0, 2, UIntPtr.Zero); keybd_event(0x12, 0, 2, UIntPtr.Zero);
        if(withShift) keybd_event(0x10, 0, 2, UIntPtr.Zero);
        keybd_event(0x11, 0, 2, UIntPtr.Zero);
    }
    public static void Position(string title, int x, int y) {
        SetThreadDpiAwarenessContext(new IntPtr(-4));
        var window = FindWindow(null, title);
        if(window == IntPtr.Zero || !SetWindowPos(window, IntPtr.Zero, x, y, 0, 0, 0x15)) throw new Exception("Could not position owned source");
    }
    public static void Close(string title, int process) {
        if(process <= 0) throw new Exception("Closing requires the owned process id");
        var target = IntPtr.Zero;
        EnumWindows((window, _) => {
            GetWindowThreadProcessId(window, out var owner);
            var name = new StringBuilder(260); GetWindowText(window, name, name.Capacity);
            if(owner == process && name.ToString() == title) target = window;
            return target == IntPtr.Zero;
        }, IntPtr.Zero);
        if(target == IntPtr.Zero || !PostMessage(target, 0x10, IntPtr.Zero, IntPtr.Zero)) throw new Exception("Could not request owned window close");
    }
    public static string Bounds(string title, int process) {
        SetThreadDpiAwarenessContext(new IntPtr(-4));
        var rectangles = new List<string>();
        EnumWindows((window, _) => {
            GetWindowThreadProcessId(window, out var owner);
            var name = new StringBuilder(260); GetWindowText(window, name, name.Capacity);
            if(name.ToString() != title || (process != 0 && owner != process) || !IsWindowVisible(window)) return true;
            GetWindowRect(window, out var outer);
            var edit = FindWindowEx(window, IntPtr.Zero, "RICHEDIT50W", null);
            var client = edit == IntPtr.Zero ? window : edit;
            GetClientRect(client, out var inner); var origin = new Point(); ClientToScreen(client, ref origin);
            rectangles.Add("{\"x\":" + origin.X + ",\"y\":" + origin.Y + ",\"width\":" + inner.Right + ",\"height\":" + inner.Bottom +
                ",\"outerX\":" + outer.Left + ",\"outerY\":" + outer.Top + ",\"outerWidth\":" + (outer.Right - outer.Left) +
                ",\"outerHeight\":" + (outer.Bottom - outer.Top) + ",\"dpi\":" + GetDpiForWindow(window) + "}");
            return true;
        }, IntPtr.Zero);
        if(rectangles.Count == 0) throw new Exception("Owned window bounds unavailable: " + title);
        return "[" + string.Join(",", rectangles) + "]";
    }
    public static void Drag(int x, int y, int endX, int endY) {
        SetThreadDpiAwarenessContext(new IntPtr(-4));
        if(!SetCursorPos(x, y)) throw new Exception("Could not position selection cursor");
        mouse_event(2, 0, 0, 0, UIntPtr.Zero); System.Threading.Thread.Sleep(80);
        SetCursorPos(endX, endY); System.Threading.Thread.Sleep(80); mouse_event(4, 0, 0, 0, UIntPtr.Zero);
    }
    public static string Monitors() {
        SetThreadDpiAwarenessContext(new IntPtr(-4));
        var monitors = new List<string>();
        EnumDisplayMonitors(IntPtr.Zero, IntPtr.Zero, (IntPtr monitor, IntPtr dc, ref Rect bounds, IntPtr parameter) => {
            var info = new MonitorInfo { Size = (uint)Marshal.SizeOf<MonitorInfo>() };
            if(!GetMonitorInfo(monitor, ref info)) throw new Exception("Monitor bounds unavailable");
            monitors.Add("{\"x\":" + bounds.Left + ",\"y\":" + bounds.Top + ",\"width\":" + (bounds.Right - bounds.Left) +
                ",\"height\":" + (bounds.Bottom - bounds.Top) + ",\"primary\":" + ((info.Flags & 1) != 0 ? "true" : "false") + "}");
            return true;
        }, IntPtr.Zero);
        if(monitors.Count == 0) throw new Exception("No active monitors");
        return "[" + string.Join(",", monitors) + "]";
    }
    static string ClassName(IntPtr window) {
        var text = new StringBuilder(256);
        if(GetClassName(window, text, text.Capacity) == 0) throw new Exception("Owned control class unavailable");
        return text.ToString();
    }
    static void ValidatePickerProcess(int process, string executable) {
        if(process <= 0 || String.IsNullOrEmpty(executable)) throw new Exception("Picker requires the owned PID and executable");
        var actual = System.Diagnostics.Process.GetProcessById(process).MainModule.FileName;
        if(!String.Equals(actual, System.IO.Path.GetFullPath(executable), StringComparison.OrdinalIgnoreCase)) throw new Exception("Picker executable mismatch");
    }
    static bool OwnedControl(IntPtr window, IntPtr dialog, int process) {
        GetWindowThreadProcessId(window, out var owner);
        return owner == process && IsWindow(window) && GetAncestor(window, 2) == dialog;
    }
    static IntPtr PickerControl(IntPtr dialog, int process, string cls, int id) {
        var matches = new List<IntPtr>();
        EnumChildWindows(dialog, (window, _) => {
            if(OwnedControl(window, dialog, process) && IsWindowVisible(window) && IsWindowEnabled(window) && GetDlgCtrlID(window) == id && ClassName(window) == cls) matches.Add(window);
            return true;
        }, IntPtr.Zero);
        if(matches.Count != 1) throw new Exception("Observed picker control is missing or ambiguous: " + cls + "/" + id);
        return matches[0];
    }
    static string ReadControl(IntPtr window) {
        var text = new StringBuilder(32768);
        if(TimedRead(window, 0xD, new UIntPtr((uint)text.Capacity), text, 2, 500, out _) == IntPtr.Zero) throw new Exception("Owned picker read timed out");
        return text.ToString();
    }
    public static Dictionary<string, object> Picker(string action, int process, string executable, string filename, int filenameId, int waitMs) {
        ValidatePickerProcess(process, executable);
        if(action == "picker-windows") {
            var windows = new List<Dictionary<string, object>>();
            EnumWindows((window, _) => {
                GetWindowThreadProcessId(window, out var owner);
                if(owner != process || !IsWindowVisible(window)) return true;
                var caption = new StringBuilder(260); GetWindowText(window, caption, caption.Capacity);
                windows.Add(new Dictionary<string, object> { {"handle", window.ToInt64()}, {"class", ClassName(window)}, {"owner", GetWindow(window, 4).ToInt64()}, {"mainCaption", caption.ToString() == "Lexique"} });
                return true;
            }, IntPtr.Zero);
            return new Dictionary<string, object> { {"pid", process}, {"windows", windows} };
        }
        var elapsed = System.Diagnostics.Stopwatch.StartNew();
        IntPtr main = IntPtr.Zero, dialog = IntPtr.Zero;
        do {
            var mains = new List<IntPtr>();
            EnumWindows((window, _) => {
                GetWindowThreadProcessId(window, out var owner);
                if(owner != process || !IsWindowVisible(window) || GetWindow(window, 4) != IntPtr.Zero) return true;
                var caption = new StringBuilder(260); GetWindowText(window, caption, caption.Capacity);
                if(caption.ToString() == "Lexique") mains.Add(window);
                return true;
            }, IntPtr.Zero);
            if(mains.Count > 1) throw new Exception("Owned main window is ambiguous");
            if(mains.Count == 1) {
                main = mains[0];
                var dialogs = new List<IntPtr>();
                EnumWindows((window, _) => {
                    GetWindowThreadProcessId(window, out var owner);
                    if(owner != process || !IsWindowVisible(window) || window == main) return true;
                    var parent = GetWindow(window, 4);
                    for(var count = 0; parent != IntPtr.Zero && count < 16; count++) {
                        if(parent == main) { dialogs.Add(window); break; }
                        parent = GetWindow(parent, 4);
                    }
                    return true;
                }, IntPtr.Zero);
                if(dialogs.Count > 1) throw new Exception("Owned modal window is ambiguous");
                if(dialogs.Count == 1) { dialog = dialogs[0]; break; }
            }
            System.Threading.Thread.Sleep(50);
        } while(elapsed.ElapsedMilliseconds < waitMs);
        if(dialog == IntPtr.Zero) throw new Exception("Owned picker was not found before its deadline");
        var controls = new List<Dictionary<string, object>>();
        EnumChildWindows(dialog, (window, _) => {
            if(OwnedControl(window, dialog, process)) controls.Add(new Dictionary<string, object> {
                {"handle", window.ToInt64()}, {"parent", GetAncestor(window, 1).ToInt64()}, {"class", ClassName(window)},
                {"id", GetDlgCtrlID(window)}, {"visible", IsWindowVisible(window)}, {"enabled", IsWindowEnabled(window)} });
            return true;
        }, IntPtr.Zero);
        var result = new Dictionary<string, object> { {"pid", process}, {"main", main.ToInt64()}, {"dialog", dialog.ToInt64()}, {"class", ClassName(dialog)}, {"controls", controls} };
        if(action == "picker-controls") return result;
        if(ClassName(dialog) != "#32770") throw new Exception("Observed modal is not the calibrated Windows file picker");
        ValidatePickerProcess(process, executable);
        var button = PickerControl(dialog, process, "Button", action == "picker-cancel" ? 2 : 1);
        result["button"] = new Dictionary<string, object> { {"handle", button.ToInt64()}, {"id", GetDlgCtrlID(button)}, {"caption", ReadControl(button)} };
        if(action == "picker-select") {
            if(filenameId == 0 || !System.IO.Path.IsPathFullyQualified(filename)) throw new Exception("Picker selection requires an observed filename control and absolute filename");
            var edit = PickerControl(dialog, process, "Edit", filenameId);
            if(TimedText(edit, 0xC, UIntPtr.Zero, filename, 2, 500, out _) == IntPtr.Zero) throw new Exception("Owned filename write timed out");
            var readback = ReadControl(edit);
            if(readback != filename) throw new Exception("Owned filename readback differs");
            result["filename"] = readback; result["filenameControlId"] = filenameId;
        }
        PeekMessage(out _, IntPtr.Zero, 0, 0, 0);
        var ownThread = GetCurrentThreadId(); var targetThread = GetWindowThreadProcessId(dialog, out _);
        var attached = ownThread != targetThread;
        if(attached && !AttachThreadInput(ownThread, targetThread, true)) throw new Exception("Could not attach owned picker input");
        IntPtr active;
        try { SetActiveWindow(dialog); active = GetActiveWindow(); }
        finally { if(attached) AttachThreadInput(ownThread, targetThread, false); }
        result["active"] = active.ToInt64();
        if(active != dialog) throw new Exception("Owned picker is not active: expected " + dialog.ToInt64() + ", actual " + active.ToInt64());
        if(!OwnedControl(button, dialog, process) || !IsWindowEnabled(button) || TimedMessage(button, 0xF5, UIntPtr.Zero, IntPtr.Zero, 2, 500, out _) == IntPtr.Zero) throw new Exception("Owned picker button delivery failed");
        for(var count = 0; IsWindow(dialog) && count < 30; count++) System.Threading.Thread.Sleep(50);
        if(IsWindow(dialog)) throw new Exception("Owned picker did not disappear");
        result["disappeared"] = true; result["elapsedMs"] = elapsed.ElapsedMilliseconds;
        return result;
    }
}
'@
if ($Action.StartsWith('picker-')) { [NativeInputFixture]::Picker($Action, $SourcePid, $SourceExecutable, $Text, $FilenameControlId, $WaitMs) | ConvertTo-Json -Depth 6 -Compress }
elseif ($Action -eq 'editor') { [NativeInputFixture]::Editor($Title, $Text, $OcrSample) }
elseif ($Action -eq 'focus') { [NativeInputFixture]::FocusSource($Title, $SourcePid, $OcrSample) }
elseif ($Action -eq 'shortcut') { [NativeInputFixture]::Shortcut($Title, $Key, $SourcePid, $WithShift, $OcrSample) }
elseif ($Action -eq 'position') { [NativeInputFixture]::Position($Title, $X, $Y) }
elseif ($Action -eq 'bounds') { [NativeInputFixture]::Bounds($Title, $SourcePid) }
elseif ($Action -eq 'monitors') { [NativeInputFixture]::Monitors() }
elseif ($Action -eq 'close') { [NativeInputFixture]::Close($Title, $SourcePid) }
else { [NativeInputFixture]::Drag($X, $Y, $EndX, $EndY) }
