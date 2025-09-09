import { PlaitElement, Point } from '@plait/core';
import { DrawTextInfo } from '../generators/text.generator';
import { ParagraphElement, StrokeStyle } from '@plait/common';

/**
 * 基础图形
 */
export enum BasicShapes {
  rectangle = 'rectangle',
  ellipse = 'ellipse',
  diamond = 'diamond',
  roundRectangle = 'roundRectangle',
  parallelogram = 'parallelogram',
  text = 'text',
  triangle = 'triangle',
  leftArrow = 'leftArrow',
  trapezoid = 'trapezoid',
  rightArrow = 'rightArrow',
  cross = 'cross',
  star = 'star',
  pentagon = 'pentagon',
  hexagon = 'hexagon',
  octagon = 'octagon',
  pentagonArrow = 'pentagonArrow',
  processArrow = 'processArrow',
  twoWayArrow = 'twoWayArrow',
  comment = 'comment',
  roundComment = 'roundComment',
  cloud = 'cloud'
}

/**
 * 流程图符号
 */
export enum FlowchartSymbols {
  process = 'process',
  decision = 'decision',
  data = 'data',
  connector = 'connector',
  terminal = 'terminal',
  manualInput = 'manualInput',
  preparation = 'preparation',
  manualLoop = 'manualLoop',
  merge = 'merge',
  delay = 'delay',
  storedData = 'storedData',
  or = 'or',
  summingJunction = 'summingJunction',
  predefinedProcess = 'predefinedProcess',
  offPage = 'offPage',
  document = 'document',
  multiDocument = 'multiDocument',
  database = 'database',
  hardDisk = 'hardDisk',
  internalStorage = 'internalStorage',
  noteCurlyRight = 'noteCurlyRight',
  noteCurlyLeft = 'noteCurlyLeft',
  noteSquare = 'noteSquare',
  display = 'display'
}

/**
 * UML 符号
 */
export enum UMLSymbols {
  actor = 'actor',
  useCase = 'useCase',
  container = 'container',
  note = 'note',
  simpleClass = 'simpleClass',
  activityClass = 'activityClass',
  branchMerge = 'branchMerge',
  port = 'port',
  package = 'package',
  combinedFragment = 'combinedFragment',
  class = 'class',
  interface = 'interface',
  object = 'object',
  component = 'component',
  componentBox = 'componentBox',
  template = 'template',
  activation = 'activation',
  deletion = 'deletion',
  assembly = 'assembly',
  providedInterface = 'providedInterface',
  requiredInterface = 'requiredInterface'
}

/**
 * 公共文字 key
 */
export enum GeometryCommonTextKeys {
  name = 'name',
  content = 'content'
}

/**
 * 所有几何图形类型联合
 */
export type GeometryShapes = BasicShapes | FlowchartSymbols | UMLSymbols;

/**
 * 泳道方向
 */
export type SwimlaneDirection = 'horizontal' | 'vertical';

/**
 * 基础几何元素
 */
export interface PlaitBaseGeometry<
  T extends string = 'geometry',
  P extends Point[] = [Point, Point],
  S extends string = GeometryShapes
> extends PlaitElement {
  type: T;
  points: P;
  shape: S;
}

/**
 * 通用几何元素
 */
export interface PlaitCommonGeometry<
  T extends string = 'geometry',
  P extends Point[] = [Point, Point],
  S extends string = GeometryShapes
> extends PlaitBaseGeometry<T, P, S> {
  fill?: string;
  strokeColor?: string;
  strokeWidth?: number;
  strokeStyle?: StrokeStyle;
  angle?: number;
  opacity?: number;
}

/**
 * 自定义几何元素
 */
export interface PlaitCustomGeometry<
  T extends string = string,
  P extends Point[] = Point[],
  S extends string = string
> extends PlaitBaseGeometry<T, P, S> {}

/**
 * 多文本几何
 */
export interface PlaitMultipleTextGeometry extends PlaitCommonGeometry {
  texts: DrawTextInfo[];
}

/**
 * 单文本几何
 */
export interface PlaitGeometry extends PlaitCommonGeometry {
  text?: ParagraphElement;
  textHeight?: number;
}

/**
 * 具体图形接口
 */
export interface PlaitRectangle extends PlaitGeometry {
  shape: BasicShapes.rectangle;
}
export interface PlaitEllipse extends PlaitGeometry {
  shape: BasicShapes.ellipse;
}
export interface PlaitDiamond extends PlaitGeometry {
  shape: BasicShapes.diamond;
}
