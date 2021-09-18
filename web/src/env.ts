const addr =
  process.env.NODE_ENV === "production"
    ? ""
    : process.env.REACT_APP_QUEST_ADDR || "";

export function questPath(path = ""): string {
  return `${addr}/${path}`;
}
