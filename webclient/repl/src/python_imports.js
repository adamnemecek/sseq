import initialize_pyodide_0 from "./python/initialize_pyodide.py";
import namespace_1 from "./python/namespace.py";
import sseq_display_2 from "./python/sseq_display.py";
import working_directory_3 from "./python/working_directory.py";
import async_js_4 from "./python/js_wrappers/async_js.py";
import crappy_multitasking_5 from "./python/js_wrappers/crappy_multitasking.py";
import filesystem_6 from "./python/js_wrappers/filesystem.py";
import messages_7 from "./python/js_wrappers/messages.py";
import __init___8 from "./python/js_wrappers/__init__.py";
import completer_9 from "./python/repl/completer.py";
import execution_10 from "./python/repl/execution.py";
import executor_11 from "./python/repl/executor.py";
import handler_decorator_12 from "./python/repl/handler_decorator.py";
import traceback_13 from "./python/repl/traceback.py";
import write_stream_14 from "./python/repl/write_stream.py";
import __init___15 from "./python/repl/__init__.py";
export const directories_to_install = ['js_wrappers', 'repl']; 
export const files_to_install = {'initialize_pyodide.py' : initialize_pyodide_0, 'namespace.py' : namespace_1, 'sseq_display.py' : sseq_display_2, 'working_directory.py' : working_directory_3, 'js_wrappers/async_js.py' : async_js_4, 'js_wrappers/crappy_multitasking.py' : crappy_multitasking_5, 'js_wrappers/filesystem.py' : filesystem_6, 'js_wrappers/messages.py' : messages_7, 'js_wrappers/__init__.py' : __init___8, 'repl/completer.py' : completer_9, 'repl/execution.py' : execution_10, 'repl/executor.py' : executor_11, 'repl/handler_decorator.py' : handler_decorator_12, 'repl/traceback.py' : traceback_13, 'repl/write_stream.py' : write_stream_14, 'repl/__init__.py' : __init___15};