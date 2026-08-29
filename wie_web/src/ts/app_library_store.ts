export interface AppMetadata {
  id: string;
  title: string;
  filename: string;
  icon?: Blob;
  addedAt: number;
}

export class AppLibraryStore {
  private constructor(private readonly db: IDBDatabase) {}

  public static open(): Promise<AppLibraryStore> {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open("wie_app_library", 1);

      request.onupgradeneeded = () => {
        request.result.createObjectStore("apps", { keyPath: "id" });
        request.result.createObjectStore("archives");
      };

      request.onsuccess = () => resolve(new AppLibraryStore(request.result));
      request.onerror = () => reject(request.error);
    });
  }

  public list(): Promise<AppMetadata[]> {
    return new Promise((resolve, reject) => {
      const request = this.db.transaction("apps", "readonly").objectStore("apps").getAll();

      request.onsuccess = () => {
        const apps = request.result as AppMetadata[];
        apps.sort((left, right) => left.addedAt - right.addedAt || left.id.localeCompare(right.id));
        resolve(apps);
      };
      request.onerror = () => reject(request.error);
    });
  }

  public getArchive(id: string): Promise<Uint8Array | undefined> {
    return new Promise((resolve, reject) => {
      const request = this.db.transaction("archives", "readonly").objectStore("archives").get(id);

      request.onsuccess = () => resolve(request.result as Uint8Array | undefined);
      request.onerror = () => reject(request.error);
    });
  }

  public add(metadata: AppMetadata, archive: Uint8Array): Promise<void> {
    return new Promise((resolve, reject) => {
      const transaction = this.db.transaction(["apps", "archives"], "readwrite");
      transaction.objectStore("apps").add(metadata);
      transaction.objectStore("archives").add(archive, metadata.id);

      transaction.oncomplete = () => resolve();
      transaction.onerror = () => reject(transaction.error);
      transaction.onabort = () => reject(transaction.error);
    });
  }

  public delete(id: string): Promise<void> {
    return new Promise((resolve, reject) => {
      const transaction = this.db.transaction(["apps", "archives"], "readwrite");
      transaction.objectStore("apps").delete(id);
      transaction.objectStore("archives").delete(id);

      transaction.oncomplete = () => resolve();
      transaction.onerror = () => reject(transaction.error);
      transaction.onabort = () => reject(transaction.error);
    });
  }
}
