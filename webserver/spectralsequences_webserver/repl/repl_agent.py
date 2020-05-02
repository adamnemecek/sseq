from message_passing_tree.prelude import *
from .console_io import ConsoleIO
from typing import Optional
from prompt_toolkit import HTML
from prompt_toolkit.patch_stdout import patch_stdout as patch_stdout_context


import logging
logger = logging.getLogger(__name__)

@subscribe_to([]) # root node.
@collect_transforms(inherit=False) # Nothing to inherit
class ReplAgent(Agent):
    def __init__(self,
        vi_mode: bool = False,
        history_filename: Optional[str] = None,
        title: Optional[str] = None
    ):
        super().__init__()
        self.executor = None
        def get_globals():
            return self.executor.get_globals()

        def get_locals():
            return self.executor.get_locals()

        # Create REPL.
        self.console_io = ConsoleIO(
            get_globals=get_globals,
            get_locals=get_locals,
            vi_mode=vi_mode,
            history_filename=history_filename,
        )
        if title:
            self.console_io.terminal_title = title
        self.patch_context : ContextManager = patch_stdout_context()

    async def start_a(self):
        with self.patch_context:
            await self.console_io.run_a()

    def set_executor(self, executor):
        if self.console_io.executor:
            self.console_io.print_formatted_text(HTML(
                "<orange>Switching executor!!</orange>"
            ), buffered=True)
        self.executor = executor
        self.console_io.executor = executor


    @transform_inbound_messages
    async def transform__debug__a(self, envelope, msg):#source, cmd, msg):
        envelope.mark_used()
        self.console_io.print_debug(".".join(cmd.part_list[1:]), msg)

    @transform_inbound_messages
    async def transform__info__a(self, envelope, msg):
        # print("consume_info", args, kwargs)
        envelope.mark_used()
        self.console_io.print_info(".".join(cmd.part_list[1:]), msg)

    @transform_inbound_messages
    async def transform__warning__a(self, envelope, msg):
        envelope.mark_used()
        self.console_io.print_warning(".".join(cmd.part_list[1:]), msg)

    @transform_inbound_messages
    async def transform__error__exception__a(self, envelope, msg,  exception):
        # do something with cmd?
        envelope.mark_used()
        self.console_io.print_exception(exception)

    @transform_inbound_messages
    async def transform__error__additional_info__a(self, envelope, msg, additional_info):
        envelope.mark_used()
        self.console_io.print_error(".".join(cmd.part_list[2:]), msg, additional_info)
