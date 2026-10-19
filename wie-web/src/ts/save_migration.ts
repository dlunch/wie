// Temporary browser save migration added 2026-10-19; remove with the startup call once obsolete.
function transaction<T>(db: IDBDatabase, stores: string[], mode: IDBTransactionMode, run: (tx: IDBTransaction) => () => T): Promise<T> {
  return new Promise((resolve, reject) => {
    const tx = db.transaction(stores, mode);
    const result = run(tx);
    tx.oncomplete = () => resolve(result());
    tx.onabort = () => reject(tx.error ?? new Error("Save migration transaction aborted"));
  });
}

function open(name: string, filesystem: boolean): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = filesystem ? indexedDB.open(name, 2) : indexedDB.open(name);
    let blocked = false;
    request.onblocked = () => {
      blocked = true;
      reject(new Error("Close other WIE tabs before migrating saves"));
    };
    request.onerror = () => reject(request.error);
    request.onsuccess = () => {
      if (blocked) request.result.close();
      else resolve(request.result);
    };
    request.onupgradeneeded = () => {
      const db = request.result;
      if (!filesystem) {
        db.createObjectStore(name);
        return;
      }
      if (!db.objectStoreNames.contains("files")) {
        db.createObjectStore("files");
        return;
      }
      const files = request.transaction!.objectStore("files");
      const keys = files.getAllKeys();
      const values = files.getAll();
      values.onsuccess = () => {
        files.clear();
        keys.result.forEach((key, index) => {
          const migrated = Array.isArray(key) ? [`migrated_aid_${key[0]}`, key[1]] : key;
          files.put(values.result[index], migrated);
        });
      };
    };
  });
}

export async function migrateSaves(pid: string, aid: string, clet: boolean, normalize: (path: string) => string | undefined): Promise<void> {
  await navigator.locks.request("wie-save-migration", async () => {
    const filesystem = await open("wie_filesystem", true);
    let records: IDBDatabase | undefined;
    try {
      if (clet && (await indexedDB.databases()).some(db => db.name === `wie_${pid}`)) {
        records = await open(`wie_${pid}`, false);
      }
      const candidates = records ? await transaction(records, [`wie_${pid}`], "readonly", tx => {
        const store = tx.objectStore(`wie_${pid}`);
        const keys = store.getAllKeys();
        const values = store.getAll();
        return () => keys.result.flatMap((key, index) => {
          if (typeof key !== "string" || !key.endsWith("1")) return [];
          const name = key.slice(0, -1);
          // Names ending in digits cannot be separated from record IDs in the legacy encoding.
          if (/\d$/.test(name) || keys.result.some(other => typeof other === "string" && other !== key
            && other.startsWith(name) && /^\d+$/.test(other.slice(name.length)))) return [];
          const path = normalize(name);
          return path === undefined ? [] : [{ key, path, data: values.result[index] as Uint8Array }];
        });
      }) : [];
      await transaction(filesystem, ["files"], "readwrite", tx => {
        const files = tx.objectStore("files");
        const cursor = files.openCursor(IDBKeyRange.bound([`migrated_aid_${aid}`], [`migrated_aid_${aid}`, []]));
        cursor.onsuccess = () => {
          const row = cursor.result;
          if (row) {
            const target = [pid, (row.key as string[])[1]];
            const existing = files.get(target);
            existing.onsuccess = () => {
              if (existing.result === undefined) {
                files.put(row.value, target);
                row.delete();
              }
              row.continue();
            };
          } else {
            for (const candidate of candidates) files.put(candidate.data, [pid, candidate.path]);
          }
        };
        return () => undefined;
      });
      // IndexedDB cannot transact across databases: commit files before deleting their sources.
      if (records && candidates.length) {
        await transaction(records, [`wie_${pid}`], "readwrite", tx => {
          for (const candidate of candidates) tx.objectStore(`wie_${pid}`).delete(candidate.key);
          return () => undefined;
        });
      }
    } finally {
      records?.close();
      filesystem.close();
    }
  });
}
