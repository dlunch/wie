import { IndexedDBStore, type Upgrade } from "./indexed_db_store";

const UPGRADES: Upgrade[] = [
  // v1: create the store.
  db => db.createObjectStore("files"),
  // v2: AID-keyed saves are unreachable from PID namespaces.
  (_, transaction) => transaction.objectStore("files").clear(),
  // v3: delete saves left under migrated_aid_ by the removed save migration.
  (_, transaction) => transaction.objectStore("files").delete(IDBKeyRange.bound(["migrated_aid_"], ["migrated_aid`"], false, true)),
];

export const openFilesystemStore = (): Promise<IndexedDBStore> => IndexedDBStore.open("wie_filesystem", "files", UPGRADES);
