export function docLink(docId: number, folderPath: string): string {
  return `/doc/${docId}?path=${encodeURIComponent(folderPath)}`;
}
