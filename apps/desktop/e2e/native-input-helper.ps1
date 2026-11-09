param(
    [ValidateSet('editor', 'focus', 'shortcut', 'position', 'bounds', 'drag', 'monitors', 'close')] [string] $Action,
    [string] $Text = 'école',
    [string] $Title = 'Lexique selection editor',
    [int] $SourcePid = 0,
    [byte] $Key = 0x4B,
    [switch] $WithShift,
    [switch] $OcrSample,
    [int] $X = 30,
    [int] $Y = 30,
    [int] $EndX = 0,
    [int] $EndY = 0
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
}
'@
if ($Action -eq 'editor') { [NativeInputFixture]::Editor($Title, $Text, $OcrSample) }
elseif ($Action -eq 'focus') { [NativeInputFixture]::FocusSource($Title, $SourcePid, $OcrSample) }
elseif ($Action -eq 'shortcut') { [NativeInputFixture]::Shortcut($Title, $Key, $SourcePid, $WithShift, $OcrSample) }
elseif ($Action -eq 'position') { [NativeInputFixture]::Position($Title, $X, $Y) }
elseif ($Action -eq 'bounds') { [NativeInputFixture]::Bounds($Title, $SourcePid) }
elseif ($Action -eq 'monitors') { [NativeInputFixture]::Monitors() }
elseif ($Action -eq 'close') { [NativeInputFixture]::Close($Title, $SourcePid) }
else { [NativeInputFixture]::Drag($X, $Y, $EndX, $EndY) }
