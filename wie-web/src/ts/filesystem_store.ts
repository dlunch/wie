import { IndexedDBStore, type Upgrade } from "./indexed_db_store";

const UPGRADES: Upgrade[] = [
  db => db.createObjectStore("files"),
  // AID-keyed saves are unreachable from PID namespaces.
  (_, transaction) => transaction.objectStore("files").clear(),
  (_, transaction) => transaction.objectStore("files").delete(IDBKeyRange.bound(["migrated_aid_"], ["migrated_aid`"], false, true)),
];

export const openFilesystemStore = (): Promise<IndexedDBStore> => IndexedDBStore.open("wie_filesystem", "files", UPGRADES);
