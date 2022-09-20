import mosaic_sdk

__doc__ = mosaic_sdk.__doc__
if hasattr(mosaic_sdk, "__all__"):
	__all__ = mosaic_sdk.__all__
	print(__all__)


client = mosaic_sdk.Client("[::]:8080")