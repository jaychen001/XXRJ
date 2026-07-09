import { useEffect, useState } from "react";
import { Archive, CheckCircle2, Trash2 } from "lucide-react";
import type { VendorLibraryRecord } from "../../domain/vendor";
import {
  deleteVendorLibrary,
  listVendorLibraries,
  setVendorLibraryEnabled,
} from "../../shared/api/vendor";

interface VendorLibraryPanelProps {
  refreshKey: number;
  onChanged: () => void;
}

export function VendorLibraryPanel({ refreshKey, onChanged }: VendorLibraryPanelProps) {
  const [libraries, setLibraries] = useState<VendorLibraryRecord[]>([]);
  const [status, setStatus] = useState("正在读取已导入样本库");
  const [busyId, setBusyId] = useState("");
  const [pendingDeleteId, setPendingDeleteId] = useState("");

  useEffect(() => {
    void loadLibraries();
  }, [refreshKey]);

  async function loadLibraries(nextStatus?: string) {
    if (!nextStatus) {
      setStatus("正在读取已导入样本库");
    }
    try {
      const records = await listVendorLibraries();
      setLibraries(records);
      setStatus(
        nextStatus ?? (records.length > 0 ? `已导入 ${records.length} 个样本库` : "还没有样本库。"),
      );
    } catch (error: unknown) {
      setStatus(error instanceof Error ? error.message : String(error));
    }
  }

  async function handleToggle(library: VendorLibraryRecord) {
    setBusyId(library.id);
    try {
      await setVendorLibraryEnabled(library.id, !library.enabled);
      await loadLibraries(library.enabled ? "样本库已停用" : "样本库已启用");
      onChanged();
    } catch (error: unknown) {
      setStatus(error instanceof Error ? error.message : String(error));
    } finally {
      setBusyId("");
    }
  }

  async function handleDelete(library: VendorLibraryRecord) {
    if (pendingDeleteId !== library.id) {
      setPendingDeleteId(library.id);
      setStatus(`再次点击确认删除 ${library.name}`);
      return;
    }
    setBusyId(library.id);
    try {
      await deleteVendorLibrary(library.id);
      setPendingDeleteId("");
      await loadLibraries("样本库已删除");
      onChanged();
    } catch (error: unknown) {
      setStatus(error instanceof Error ? error.message : String(error));
    } finally {
      setBusyId("");
    }
  }

  return (
    <section className="vendor-inline-panel" aria-label="已导入样本库">
      <header>
        <div>
          <h4>已导入样本库</h4>
          <p>停用后不参与型号匹配，删除会移除该库下的型号。</p>
        </div>
        <Archive size={17} aria-hidden="true" />
      </header>

      <span className="vendor-inline-status" role="status">
        {status}
      </span>

      <div className="vendor-library-list">
        {libraries.map((library) => (
          <article className="vendor-library-card" key={library.id}>
            <div>
              <strong>{library.name}</strong>
              <span>
                {library.componentType} · {library.modelCount} 个型号
              </span>
            </div>
            <span
              className={
                library.enabled
                  ? "vendor-library-card__state vendor-library-card__state--enabled"
                  : "vendor-library-card__state"
              }
            >
              <CheckCircle2 size={14} aria-hidden="true" />
              {library.enabled ? "启用" : "停用"}
            </span>
            <button
              className="vendor-inline-button"
              type="button"
              disabled={busyId === library.id}
              aria-label={`${library.enabled ? "停用" : "启用"}${library.name}`}
              onClick={() => void handleToggle(library)}
            >
              {library.enabled ? "停用" : "启用"}
            </button>
            <button
              className="vendor-inline-button vendor-inline-button--danger"
              type="button"
              disabled={busyId === library.id}
              aria-label={
                pendingDeleteId === library.id ? `确认删除${library.name}` : `删除${library.name}`
              }
              onClick={() => void handleDelete(library)}
            >
              <Trash2 size={14} aria-hidden="true" />
              {pendingDeleteId === library.id ? "确认删除" : "删除"}
            </button>
          </article>
        ))}
        {libraries.length === 0 ? <p className="vendor-library-empty">还没有样本库。</p> : null}
      </div>
    </section>
  );
}
