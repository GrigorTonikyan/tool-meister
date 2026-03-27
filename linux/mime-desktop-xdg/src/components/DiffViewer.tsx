import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Eye, Clock, RotateCcw } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { ScrollArea } from "@/components/ui/scroll-area"

interface DiffViewerProps {
  filePath?: string | null;
  diffId?: string | null;
  onClose: () => void;
}

export function DiffViewer({ filePath, diffId, onClose }: DiffViewerProps) {
  const [content, setContent] = useState<string>("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (filePath) {
      setLoading(true);
      invoke<string>("read_file", { path: filePath })
        .then(setContent)
        .catch((e) => setContent(`Error reading file:\n${e}`))
        .finally(() => setLoading(false));
    } else if (diffId) {
      setLoading(true);
      invoke<string>("get_snapshot_diff", { commitId: diffId })
        .then(setContent)
        .catch((e) => setContent(`Error getting diff:\n${e}`))
        .finally(() => setLoading(false));
    } else {
      setContent("");
    }
  }, [filePath, diffId]);

  const isOpen = !!filePath || !!diffId;
  const title = filePath ? "File Viewer" : "Snapshot Changes";
  const subTitle = filePath || diffId;

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="max-w-5xl h-[85vh] flex flex-col p-0 overflow-hidden bg-zinc-950 border-zinc-800 shadow-2xl rounded-2xl">
        <DialogHeader className="p-6 bg-zinc-900/50 backdrop-blur-xl border-b border-zinc-800">
          <DialogTitle className="text-xl font-bold text-zinc-100 flex items-center gap-2">
            {filePath ? <Eye className="w-5 h-5 text-indigo-400" /> : <Clock className="w-5 h-5 text-emerald-400" />}
            {title}
          </DialogTitle>
          <DialogDescription className="text-xs font-mono text-zinc-500 break-all truncate mt-1">
            {subTitle}
          </DialogDescription>
        </DialogHeader>
        
        <div className="flex-1 overflow-hidden p-6 bg-linear-to-b from-zinc-950 to-black">
          <ScrollArea className="h-full w-full rounded-xl border border-zinc-800/50 p-6 bg-zinc-900/20 font-mono text-sm shadow-inner group relative">
            <div className="absolute inset-0 bg-indigo-500/5 blur-3xl rounded-full opacity-30 group-hover:opacity-100 transition-opacity duration-1000"></div>
            {loading ? (
              <div className="flex flex-col items-center justify-center h-full animate-pulse text-indigo-400/50 gap-3">
                <RotateCcw className="w-8 h-8 animate-spin" />
                <span className="font-bold tracking-widest text-[10px] uppercase">Retrieving Changes...</span>
              </div>
            ) : content ? (
              <pre className="relative whitespace-pre font-mono text-indigo-100/90 leading-relaxed">
                {content.split('\n').map((line, i) => (
                  <div key={i} className={`px-2 rounded-sm ${
                    line.startsWith('+') ? 'bg-emerald-500/10 text-emerald-400' : 
                    line.startsWith('-') ? 'bg-red-500/10 text-red-400' : ''
                  }`}>
                    {line || '\u00A0'}
                  </div>
                ))}
              </pre>
            ) : (
                <div className="flex items-center justify-center h-full text-zinc-600 italic">No content or empty diff.</div>
            )}
          </ScrollArea>
        </div>
      </DialogContent>
    </Dialog>
  );
}
