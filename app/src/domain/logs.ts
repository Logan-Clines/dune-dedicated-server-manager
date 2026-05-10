export type LogSegment = {
  text: string;
  className?: string;
  style?: Record<string, string | number>;
};

const ansi256Palette: Record<number, string> = {
  196: "#ff5f5f",
  202: "#ff5f00",
  208: "#ff8700",
  214: "#ffaf00",
  220: "#ffd700",
  226: "#ffff00",
  82: "#5fff00",
  46: "#00ff00",
  51: "#00ffff",
  45: "#00d7ff",
  39: "#00afff",
  117: "#87d7ff",
  141: "#af87ff",
  213: "#ff87ff",
  244: "#808080",
  250: "#bcbcbc"
};

export function visibleLogLines(lines: string[], filter: string) {
  const trimmed = filter.trim().toLowerCase();
  if (!trimmed) return lines;
  return lines.filter((line) => line.toLowerCase().includes(trimmed));
}

export function parseAnsiLine(line: string): LogSegment[] {
  const normalized = line.replace(/\u001b\[/g, "[");
  const pattern = /\[(\d+(?:;\d+)*)m/g;
  const segments: LogSegment[] = [];
  let lastIndex = 0;
  let currentStyle: Record<string, string | number> = {};
  let currentClass = logSeverityClass(normalized);
  let match: RegExpExecArray | null;

  while ((match = pattern.exec(normalized))) {
    if (match.index > lastIndex) {
      segments.push({
        text: normalized.slice(lastIndex, match.index),
        className: currentClass,
        style: Object.keys(currentStyle).length ? { ...currentStyle } : undefined
      });
    }
    const parsed = applyAnsiCodes(match[1], currentStyle, currentClass);
    currentStyle = parsed.style;
    currentClass = parsed.className;
    lastIndex = pattern.lastIndex;
  }

  segments.push({
    text: normalized.slice(lastIndex),
    className: currentClass,
    style: Object.keys(currentStyle).length ? currentStyle : undefined
  });

  return segments.filter((segment) => segment.text.length > 0);
}

function applyAnsiCodes(
  rawCodes: string,
  style: Record<string, string | number>,
  className?: string
): { style: Record<string, string | number>; className?: string } {
  const codes = rawCodes.split(";").map((code) => Number(code));
  let nextStyle = { ...style };
  let nextClass = className;

  for (let index = 0; index < codes.length; index += 1) {
    const code = codes[index];
    if (code === 0) {
      nextStyle = {};
      nextClass = undefined;
    } else if (code === 1) {
      nextStyle.fontWeight = 800;
    } else if (code === 2) {
      nextStyle.opacity = 0.72;
    } else if (code === 3) {
      nextStyle.fontStyle = "italic";
    } else if (code === 22) {
      delete nextStyle.fontWeight;
      delete nextStyle.opacity;
    } else if (code === 23) {
      delete nextStyle.fontStyle;
    } else if (code === 39) {
      delete nextStyle.color;
    } else if (code >= 30 && code <= 37) {
      const color = basicAnsiColor(code - 30);
      if (color) nextStyle.color = color;
    } else if (code >= 90 && code <= 97) {
      const color = basicAnsiColor(code - 90, true);
      if (color) nextStyle.color = color;
    } else if (code === 38 && codes[index + 1] === 5 && Number.isFinite(codes[index + 2])) {
      const color = ansi256Color(codes[index + 2]);
      if (color) nextStyle.color = color;
      index += 2;
    }
  }

  return { style: nextStyle, className: nextClass };
}

function basicAnsiColor(index: number, bright = false) {
  const normal = ["#2d3436", "#ff7675", "#55efc4", "#fdcb6e", "#74b9ff", "#a29bfe", "#81ecec", "#dfe6e9"];
  const intense = ["#636e72", "#ff8f87", "#78ffd6", "#ffe08a", "#9bd1ff", "#c0b7ff", "#a5fff4", "#ffffff"];
  return (bright ? intense : normal)[index] ?? undefined;
}

function ansi256Color(index: number) {
  if (ansi256Palette[index]) return ansi256Palette[index];
  if (index >= 16 && index <= 231) {
    const value = index - 16;
    const red = Math.floor(value / 36);
    const green = Math.floor((value % 36) / 6);
    const blue = value % 6;
    return `rgb(${ansiCube(red)}, ${ansiCube(green)}, ${ansiCube(blue)})`;
  }
  if (index >= 232 && index <= 255) {
    const shade = 8 + (index - 232) * 10;
    return `rgb(${shade}, ${shade}, ${shade})`;
  }
  return undefined;
}

function ansiCube(value: number) {
  return value === 0 ? 0 : 55 + value * 40;
}

function logSeverityClass(line: string) {
  const lower = line.toLowerCase();
  if (lower.includes("[error]") || lower.includes(" error ")) return "log-severity-error";
  if (lower.includes("[warning]") || lower.includes(" warn")) return "log-severity-warning";
  if (lower.includes("[info]") || lower.includes(" info")) return "log-severity-info";
  return undefined;
}
