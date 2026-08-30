export class IndexedDBStore {
  private db: IDBDatabase;
  private store_name: string;

  private constructor(db: IDBDatabase, store_name: string) {
    this.db = db;
    this.store_name = store_name;
  }

  public static open(db_name: string, store_name: string): Promise<IndexedDBStore> {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(db_name);

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;
        if (!db.objectStoreNames.contains(store_name)) {
          db.createObjectStore(store_name);
        }
      };

      request.onsuccess = (event) => {
        resolve(new IndexedDBStore((event.target as IDBOpenDBRequest).result, store_name));
      };

      request.onerror = (event) => {
        reject((event.target as IDBOpenDBRequest).error);
      };
    });
  }

  public get_all_keys(): Promise<IDBValidKey[]> {
    return new Promise((resolve, reject) => {
      const transaction = this.db.transaction(this.store_name, "readonly");
      const store = transaction.objectStore(this.store_name);
      const request = store.getAllKeys();

      request.onsuccess = () => {
        resolve(request.result);
      };

      request.onerror = () => {
        reject(request.error);
      };
    });
  }

  public get(key: IDBValidKey): Promise<Uint8Array | undefined> {
    return new Promise((resolve, reject) => {
      const transaction = this.db.transaction(this.store_name, "readonly");
      const store = transaction.objectStore(this.store_name);
      const request = store.get(key);

      request.onsuccess = () => {
        resolve(request.result as Uint8Array | undefined);
      };

      request.onerror = () => {
        reject(request.error);
      };
    });
  }

  public set(key: IDBValidKey, data: Uint8Array): Promise<void> {
    return new Promise((resolve, reject) => {
      const transaction = this.db.transaction(this.store_name, "readwrite");
      const store = transaction.objectStore(this.store_name);
      const request = store.put(data, key);

      request.onsuccess = () => {
        resolve();
      };

      request.onerror = () => {
        reject(request.error);
      };
    });
  }

  public delete(key: IDBValidKey): Promise<void> {
    return new Promise((resolve, reject) => {
      const transaction = this.db.transaction(this.store_name, "readwrite");
      const store = transaction.objectStore(this.store_name);
      const request = store.delete(key);

      request.onsuccess = () => {
        resolve();
      };

      request.onerror = () => {
        reject(request.error);
      };
    });
  }
}
