import asyncio


from message_passing_tree.prelude import *

from spectralsequence_chart import ChartAgent


@subscribe_to("*")
@collect_handlers(inherit = True)
class InteractiveChartAgent(ChartAgent):
    modes = {}
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.response_event = asyncio.Event()
        self.mode = NoMode
        self._interact_source = None
        self._differential_source = None
        self._extension_source = None
        self._extension_prev_color_default = "black"
        self.x_range = [0, 50]
        self.y_max = [0, 12]

    @handle_inbound_messages
    async def handle__click__a(self, envelope, x, y, chart_class=None):
        envelope.mark_used()
        if chart_class is not None:
            chart_class = self.sseq._classes[chart_class["uuid"]]
        self.schedule_coroutine(self.mode.handle_click_a(self, x, y, chart_class))

    async def prompt_for_class_name_a(self, c):
        name = await self.prompt_a(msg="Name?", default=c.name)
        if name is None:
            return
        c.name = name
        await self.update_a()

    async def prompt_a(self, msg, default):
        await self.send_message_outward_a("interact.prompt", *arguments(msg=msg, default=default))
        await self.response_event.wait()
        [args, kwargs] = self.response_event.result
        self.response_event.clear()
        return kwargs["result"]

    async def prompt_for_colors_a(self, c):
        if hasattr(c.node_list[0], "color"):
            default = c.node_list[0].color
        else:
            default = "black"
        result = await self.prompt_a(f"New color for {c}?", default)
        if result is not None:
            c.set_field("color", result)
        for e in c._edges:
            s = e.source
            t = e.target
            if hasattr(e, "color"):
                default = e.color
            else:
                default = "black"            
            result = await self.prompt_a(
                f"New color for edge {s} -- {t}?", 
                default
            )
            if result is not None:
                e.color = result
        await self.update_a()


    async def make_client_set_mode_info_a(self, info):
        await self.send_message_outward_a("interact.mode.set_info", *arguments(info = info))

    @handle_inbound_messages
    async def handle__interact__mode__set__a(self, envelope,  *args, **kwargs):
        envelope.mark_used()
        new_mode = kwargs["mode"]
        if new_mode in InteractiveChartAgent.modes:
            self.mode = InteractiveChartAgent.modes[new_mode]
        else:
            raise RuntimeError(f"""Unknown mode "{new_mode}".""")

    @handle_inbound_messages
    async def handle__interact__mode__a(self, envelope, *args, **kwargs):
        envelope.mark_used()
        cmd = envelope.msg.cmd
        f = getattr(self.mode, "handle__" + "__".join(cmd.part_list[2:]) + "__a", None)
        if f is None:
            raise RuntimeError(f"Invalid mode command {cmd.str}.")
        else:
            self.schedule_coroutine(f(self, *args, **kwargs))
            

    @handle_inbound_messages
    async def handle__interact__result__a(self, envelope, *args, **kwargs):
        envelope.mark_used()
        self.response_event.result = [args, kwargs]
        self.response_event.set()

    def add_class_on_click(self, x, y):
        rx = round(x)
        ry = round(y)
        threshold = 0.3
        if abs(x-rx) < threshold and abs(y-ry) < threshold:
            return self.sseq.add_class(rx, ry)

def register_mode(cls):
    InteractiveChartAgent.modes[cls.__name__] = cls
    return cls

class Mode:
    def __init__(self):
        raise RuntimeError("Currently I don't see any reason to instantiate Mode or its subclasses")

    async def handle_click_a(x, y, chart_class=None):
        raise RuntimeError("Override me in a subclass.")

    async def handle__cancel__a(self):
        return


@register_mode
class NoMode(Mode):
    async def handle_click_a(self, x, y, c=None):
        pass


@register_mode
class AddClassMode(Mode):
    async def handle_click_a(self, x, y, c=None):
        if c is None:
            c = self.add_class_on_click(x, y)
            if not c:
                return
            if self._interact_source is not None:
                self.sseq.add_structline(self._interact_source, c)
            self._interact_source = c
            await self.make_client_set_mode_info_a(f"""Current source: "{c}".""")
            await self.update_a()
        else:
            await self.prompt_for_class_name_a(c)
    
    async def handle__cancel__a(self):
        self._interact_source = None
        await self.make_client_set_mode_info_a("")

@register_mode
class AddEdgeMode(Mode):
    async def handle_click_a(self, x, y, c=None):
        if c is None:
            return
        if self._interact_source is None:
            self._interact_source = c
            await self.send_message_outward_a("interact.mode.set_info", *arguments(info = f"""Current source: "{c.name}"."""))
        else:
            self.sseq.add_structline(self._interact_source, c)
            self._interact_source = None
            await self.send_message_outward_a("interact.mode.set_info", *arguments(info=""))
            await self.update_a()

    async def handle__cancel__a(self):
        self._interact_source = None
        await self.send_message_outward_a("interact.mode.set_info", *arguments(info = f""""""))



@register_mode
class NameClassMode(Mode):
    async def handle_click_a(self, x, y, c=None):
        if c is None:
            return
        self.schedule_coroutine(self.prompt_for_class_name_a(c, x, y))

@register_mode
class ColorMode(Mode):
    async def handle_click_a(self, x, y, c=None):
        if c is None:
            return
        self.schedule_coroutine(self.prompt_for_colors_a(c))


@register_mode
class AddExtensionMode(Mode):
    async def handle_click_a(self, x, y, c=None):
        if c is None:
            return
        if self._extension_source is None:
            print("set_source")
            self._extension_source = c
            await self.send_message_outward_a("interact.mode.set_info", *arguments(info = f"""Current source: "{c}"."""))
        else:
            s = self._extension_source
            t = c
            if t.x - s.x not in [0, 1, 3]:
                print("invalid")
                return
            if t.y - s.y < 2:
                print("invalid")
                return                
            print("success")
            color = await self.prompt_a(msg="Color?", default=self._extension_prev_color_default)
            self._extension_prev_color_default = color
            e = self.sseq.add_extension(s, t)
            e.color = color
            self._extension = e
            self._extension_source = None
            await self.update_a()
            await self.send_message_outward_a("interact.mode.extension.adjust_bend", *arguments())

            # await self.update_a()

    async def handle__extension__adjust_bend__a(self, delta):
        e = self._extension
        bend = getattr(e, "bend", 0)
        bend += delta
        e.bend = bend
        await self.update_a()

    async def handle__cancel__a(self):
        print("unset_source")
        self._extension_source = None
        await self.send_message_outward_a("interact.mode.set_info", *arguments(info = f""""""))



@register_mode
class AddDifferentialMode(Mode):
    async def handle_click_a(self, x, y, c=None):
        if c is None:
            return
        if self._differential_source is None:
            print("set_source")
            self._differential_source = c
            await self.send_message_outward_a("interact.mode.set_info", *arguments(info = f"""Current source: "{c}"."""))
        else:
            s = self._differential_source
            t = c
            if s.x != t.x + 1:
                print("invalid")
                return
            print("success")
            page = t.y - s.y
            d = self.sseq.add_differential(page, s, t)
            d.color = "blue"
            self._differential_source = None
            await self.send_message_outward_a("interact.mode.set_info", *arguments(info=""))
            await self.update_a()

    async def handle__cancel__a(self):
        print("unset_source")
        self._differential_source = None
        await self.send_message_outward_a("interact.mode.set_info", *arguments(info = f""""""))


@register_mode
class NudgeClassMode(Mode):
    async def handle_click_a(self, x, y, c=None):
        print("handle_click",x,y,c)
        if c is None:
            return
        self._nudge_class = c
        await self.send_message_outward_a("interact.mode.set_info", *arguments(info = f"""Nudging class: "{c}"."""))

    async def handle__cancel__a(self):
        self._nudge_class = None
        await self.send_message_outward_a("interact.mode.set_info", *arguments(info = f""""""))
    
    async def handle__nudge_class__a(self, x, y):
        c = self._nudge_class
        if c:
            print("nudge", x, y)
            x_nudge = getattr(c, "x_nudge", 0)
            y_nudge = getattr(c, "y_nudge", 0)
            x_nudge += x
            y_nudge += y
            c.x_nudge = x_nudge
            c.y_nudge = y_nudge
            await self.update_a()