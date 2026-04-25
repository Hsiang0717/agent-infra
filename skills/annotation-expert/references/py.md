# Python (Docstrings + Type Hints)

## Example Module
```python
"""
@module DataProcessor
@see Pre-processing logic in {@link sanitizer.py}
"""

def process_batch(data: List[dict]) -> bool:
    """
    Process raw JSON data.
    @modifies GlobalCache - updates processing status
    @throws ValidationError - if data schema mismatch
    @context Delay added to prevent API rate limiting for X11 driver
    """
    ...
```
