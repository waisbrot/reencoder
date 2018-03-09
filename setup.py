from setuptools import setup, find_packages
from codecs import open  # To use a consistent encoding
from os import path

here = path.abspath(path.dirname(__file__))

# Get the long description from the README file
with open(path.join(here, 'README.rst'), encoding='utf-8') as f:
    long_description = f.read()

setup(
    name='reencoder',
    version='1.0.0',
    description='ffmpeg wrapper for reencoding video into smaller files',
    long_description=long_description,
    url='https://github.com/waisbrot/reencoder',
    author='waisbrot',
    author_email='code@waisbrot.net',
    classifiers=[
        'Development Status :: 4 - Beta',
        'Environment :: Console',
        'Intended Audience :: End Users/Desktop',
        'License :: OSI Approved :: GNU Affero General Public License v3 or later (AGPLv3+)',
        'Natural Language :: English',
        'Operating System :: POSIX',
        'Programming Language :: Python :: 3 :: Only',
        'Topic :: Multimedia :: Video :: Conversion',
        'Topic :: Utilities'
    ],
    keywords='video encode ffmpeg',
    packages=find_packages(exclude=['contrib', 'docs', 'tests']),

    # This field lists other packages that your project depends on to run.
    # Any package you put here will be installed by pip when your project is
    # installed, so they must be valid existing projects.
    #
    # For an analysis of "install_requires" vs pip's requirements files see:
    # https://packaging.python.org/en/latest/requirements.html
    install_requires=['progressbar2>=3.34.3,<4.0.0'],
    python_requires='>=3',
    setup_requires=['pytest-runner'],
    extras_require={
        'dev': ['check-manifest'],
        'test': ['pytest', 'coverage', 'pytest-cov'],
    },
    package_data={
    },
    entry_points={
        'console_scripts': [
            'reencode=reencode:main',
        ],
    },
)
