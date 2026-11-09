param(
    [ValidateSet('editor', 'focus', 'shortcut')] [string] $Action,
    [string] $Text = 'école',
    [string] $Title = 'Lexique selection editor',
    [int] $SourcePid = 0,
    [byte] $Key = 0x4B,
    [switch] $WithShift
)
$ErrorActionPreference = 'Stop'
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class NativeInputFixture {
    [StructLayout(LayoutKind.Sequential)] public struct Point { public int X, Y; }
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
    public static void Editor(string title, string text) {
        var window = CreateWindowEx(0, "STATIC", title, 0x10CF0000, 30, 30, 900, 430, IntPtr.Zero, IntPtr.Zero, IntPtr.Zero, IntPtr.Zero);
        if (window == IntPtr.Zero) throw new Exception("Could not create owned editor");
        if(LoadLibrary("Msftedit.dll") == IntPtr.Zero) throw new Exception("Windows RichEdit is unavailable");
        var edit = CreateWindowEx(0x200, "RICHEDIT50W", text, 0x50301004, 10, 10, 850, 350, window, IntPtr.Zero, IntPtr.Zero, IntPtr.Zero);
        if(edit == IntPtr.Zero) throw new Exception("Could not create owned RichEdit control");
        SetForegroundWindow(window); SetFocus(edit); SendMessage(edit, 0xB1, IntPtr.Zero, new IntPtr(text.Length));
        while (IsWindow(window) && GetMessage(out var message, IntPtr.Zero, 0, 0) > 0) {
            TranslateMessage(ref message); DispatchMessage(ref message);
        }
    }
    public static void FocusSource(string title, int process) {
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
        if(edit != IntPtr.Zero) SendMessage(edit, 0xB1, IntPtr.Zero, new IntPtr(-1));
        System.Threading.Thread.Sleep(120);
        if (GetForegroundWindow() != window) {
            GetWindowThreadProcessId(GetForegroundWindow(), out var foregroundProcess);
            throw new Exception("Owned source " + sourceProcess + " lost foreground to " + foregroundProcess);
        }
    }
    public static void Shortcut(string title, byte key, int process, bool withShift) {
        FocusSource(title, process);
        keybd_event(0x11, 0, 0, UIntPtr.Zero);
        if(withShift) keybd_event(0x10, 0, 0, UIntPtr.Zero);
        keybd_event(0x12, 0, 0, UIntPtr.Zero); keybd_event(key, 0, 0, UIntPtr.Zero);
        keybd_event(key, 0, 2, UIntPtr.Zero); keybd_event(0x12, 0, 2, UIntPtr.Zero);
        if(withShift) keybd_event(0x10, 0, 2, UIntPtr.Zero);
        keybd_event(0x11, 0, 2, UIntPtr.Zero);
    }
}
'@
if ($Action -eq 'editor') { [NativeInputFixture]::Editor($Title, $Text) }
elseif ($Action -eq 'focus') { [NativeInputFixture]::FocusSource($Title, $SourcePid) }
else { [NativeInputFixture]::Shortcut($Title, $Key, $SourcePid, $WithShift) }
