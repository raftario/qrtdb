import React, { useEffect, useRef, useState } from "react";
import {
  message,
  Col,
  Input,
  List,
  Row,
  Select,
  Space,
  Tag,
  Typography,
} from "antd";
import { useDebounce, useList } from "react-use";
import Fuse from "fuse.js";
import * as date from "date-fns";
import { questPath } from "./env";

type LogLevel = "Verbose" | "Debug" | "Info" | "Warn" | "Error" | "Fatal";

interface LogMessage {
  timestamp: Date;
  level: LogLevel;
  message: string;
}

type SearchType = "fuzzy" | "regex";

function filterLogs(
  logs: LogMessage[],
  search: string,
  searchType: SearchType,
) {
  if (!search) {
    return logs;
  }

  if (searchType === "fuzzy") {
    const fuse = new Fuse(logs, { keys: ["message"] });
    return fuse.search(search).map((r) => r.item);
  }

  try {
    const regex = new RegExp(search);
    return logs.filter((l) => regex.test(l.message));
  } catch (_) {
    return logs.filter((l) => l.message.includes(search));
  }
}

const LogLevelTag: React.FC<{ level: LogLevel }> = ({ level }) => {
  let color: string | undefined = undefined;
  switch (level) {
    case "Info":
      color = "blue";
      break;
    case "Warn":
      color = "gold";
      break;
    case "Error":
      color = "red";
      break;
    case "Fatal":
      color = "#f00";
      break;
  }

  return (
    <Tag color={color} style={{ width: 64 }}>
      {level}
    </Tag>
  );
};

const LogItem: React.FC<{ message: LogMessage }> = ({ message }) => (
  <Space style={{ width: "100%" }}>
    <Typography.Text code strong>
      {date.format(message.timestamp, "HH:mm:ss.SSS")}
    </Typography.Text>
    <LogLevelTag level={message.level} />
    <Typography.Text>{message.message}</Typography.Text>
  </Space>
);

const Logs: React.FC = () => {
  const bottom = useRef<HTMLDivElement>(null);

  const [source, setSource] = useState(new EventSource(questPath("logs")));
  useEffect(() => {
    const onError = () => {
      source.close();

      let delay = 5;
      const showMessage = () =>
        message.error({
          content: `Disconnected, retrying${delay ? ` in ${delay}` : ""}...`,
          key: "logs",
          duration: 6,
        });

      const interval = setInterval(() => {
        delay -= 1;
        showMessage();

        if (delay === 0) {
          clearInterval(interval);
          setSource(new EventSource(questPath("logs")));
        }
      }, 1000);
      showMessage();
    };

    if (source.readyState === EventSource.CLOSED) {
      onError();
    } else {
      source.onerror = onError;
    }
  }, [source, setSource]);

  const [logs, { push }] = useList<LogMessage>();
  useEffect(() => {
    source.onmessage = (e) => {
      const { timestamp, level, message } = JSON.parse(e.data);
      push({ timestamp: new Date(timestamp), level, message });
      bottom.current?.scrollIntoView({ behavior: "smooth", block: "end" });
    };
    return () => {
      source.onmessage = null;
    };
  }, [source, push, bottom]);

  const [searchType, setSearchType] = useState<SearchType>("fuzzy");
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState(search);
  useDebounce(
    () => {
      setDebouncedSearch(search);
      bottom.current?.scrollIntoView({ behavior: "smooth", block: "end" });
    },
    250,
    [search],
  );

  const searchTypeSelect = (
    <Select value={searchType} onChange={(v) => setSearchType(v)}>
      <Select.Option value="fuzzy">Fuzzy</Select.Option>
      <Select.Option value="regex">Regex</Select.Option>
    </Select>
  );

  return (
    <>
      <Row style={{ overflow: "auto", height: "calc(100% - 32px)" }}>
        <Col span={24}>
          <List
            size="small"
            dataSource={filterLogs(logs, debouncedSearch, searchType)}
            renderItem={(item) => (
              <List.Item>
                <LogItem message={item} />
              </List.Item>
            )}
            locale={{ emptyText: "‎ " }}
          />
          <div ref={bottom} />
        </Col>
      </Row>
      <Row align="bottom">
        <Col span={24}>
          <Input
            value={search}
            onChange={(v) => setSearch(v.target.value)}
            addonBefore={searchTypeSelect}
            placeholder="Search"
          />
        </Col>
      </Row>
    </>
  );
};
export default Logs;
