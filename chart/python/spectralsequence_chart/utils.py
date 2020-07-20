import json
from typing import Tuple, Any, Dict, Union, cast, List

def stringifier(obj : Any) -> Union[str, Dict[str, Any]]:
    if hasattr(obj, "to_json"):
        return obj.to_json()
    elif hasattr(obj, "__dict__"):
        return obj.__dict__
    else:
        return str(obj)

# Only to make typechecker happy...
class Serializable:
    @staticmethod
    def from_json(json : Dict[str, Any]):
        return Serializable()


class JSON:
    @staticmethod
    def stringify(obj : Any):
        return json.dumps(obj, default=stringifier)

    @staticmethod
    def parse(json_str : str) -> Any:
        return json.loads(json_str, object_hook = JSON.parser_object_hook )

    @staticmethod
    def parser_object_hook(json_dict : Dict[str, Any]) -> Any:
        JSON.ensure_types_are_initialized()
        if "type" not in json_dict:
            return json_dict
        return JSON.types[json_dict["type"]].from_json(json_dict)

    types : Dict[str, Serializable]
    @staticmethod
    def ensure_types_are_initialized():
        if hasattr(JSON, "types"):
            return
        from .chart import (SseqChart, ChartClass, ChartStructline, ChartDifferential, ChartExtension)
        from .helper_types import PageProperty
        JSON.types = { t.__name__ : cast(Serializable, t) for t in [
            SseqChart,
            ChartClass, ChartStructline, ChartDifferential, ChartExtension,
            PageProperty
        ]}


def replace_keys(d : Any, replace_keys : List[Tuple[str, str]]):
    for (key, replacement) in replace_keys:
        if hasattr(d, key):
            setattr(d, replacement, getattr(d, key))
            delattr(d, key)

def reverse_replace_keys(d : Any, replace_keys : List[Tuple[str, str]]):
    for (replacement, key) in replace_keys:
        if hasattr(d, key):
            setattr(d, replacement, getattr(d, key))
            delattr(d, key)

def arguments(*args : Any, **kwargs : Any) -> Tuple[Tuple, Dict[str, Any]]:
    return (args, kwargs)