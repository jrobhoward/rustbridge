package rustbridge

/*
#include <stdint.h>
*/
import "C"

import (
	"log/slog"
	"sync"
	"unsafe"
)

// LogHandler is a function that receives log messages from the plugin.
// It is called from a CGo callback, so it must not block for extended periods.
type LogHandler func(level LogLevel, target string, message string)

var (
	globalLogHandler LogHandler
	logHandlerMu     sync.RWMutex
)

func setGlobalLogHandler(handler LogHandler) {
	logHandlerMu.Lock()
	globalLogHandler = handler
	logHandlerMu.Unlock()
}

//export goLogCallback
func goLogCallback(level C.uint8_t, target *C.char, message *C.uint8_t, messageLen C.size_t) {
	defer func() {
		recover()
	}()

	logHandlerMu.RLock()
	handler := globalLogHandler
	logHandlerMu.RUnlock()

	if handler == nil {
		return
	}

	goTarget := C.GoString(target)
	goMessage := C.GoStringN((*C.char)(unsafe.Pointer(message)), C.int(messageLen))

	handler(LogLevel(level), goTarget, goMessage)
}

// SlogLogHandler returns a LogHandler that forwards messages to a slog.Logger.
func SlogLogHandler(logger *slog.Logger) LogHandler {
	return func(level LogLevel, target string, message string) {
		var slogLevel slog.Level
		switch level {
		case LogLevelTrace:
			slogLevel = slog.LevelDebug - 4
		case LogLevelDebug:
			slogLevel = slog.LevelDebug
		case LogLevelInfo:
			slogLevel = slog.LevelInfo
		case LogLevelWarn:
			slogLevel = slog.LevelWarn
		case LogLevelError:
			slogLevel = slog.LevelError
		default:
			return
		}

		logger.LogAttrs(nil, slogLevel, message, slog.String("target", target))
	}
}
